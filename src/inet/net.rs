use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use super::{
    arena::ArenaPtrIter,
    cell::{Cell, CellPtr},
    equation::{
        Equation, EquationBuilder, EquationDisplay, EquationPtr, Equations, EquationsDisplay,
    },
    heap::{Heap, CellDisplay},
    symbol::{SymbolBook, SymbolPtr},
    term::{TermFamily, TermPtr},
    var::{Var, VarPtr},
};

#[derive(Debug, Copy, Clone)]
pub struct NetF {}
impl TermFamily for NetF {
    type BoundStore = NetStore;
    type FreeStore = NetStore;

    fn display_store(
        f: &mut std::fmt::Formatter<'_>,
        symbols: &SymbolBook,
        heap: &Heap<Self>,
        var: &Var<NetF>,
        index: usize,
    ) -> std::fmt::Result {
        match var {
            Var::Bound(store) => match store.get_cell_ptr() {
                Some(cell_ptr) => write!(f, "x.{}[={}]", index, heap.display_cell(symbols, cell_ptr)),
                None => write!(f, "x.{}", index),
            },
            Var::Free(_) => write!(f, "_.{}", index),
        }
    }
}

#[derive(Debug)]
pub struct NetStore(AtomicU32);

impl NetStore {
    const NULL: u32 = u32::MAX;

    pub fn get_cell_ptr(&self) -> Option<CellPtr> {
        let value = self.0.load(Ordering::SeqCst);
        if value != Self::NULL {
            Some(CellPtr::from(value))
        } else {
            return None;
        }
    }

    pub fn get_or_set(&self, cell_ptr: CellPtr) -> Option<CellPtr> {
        let old_value = self.0.swap(cell_ptr.get_ptr(), Ordering::Acquire);
        if old_value != Self::NULL {
            if old_value != cell_ptr.get_ptr() {
                return Some(CellPtr::from(old_value));
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
}

impl Default for NetStore {
    fn default() -> Self {
        Self(AtomicU32::new(Self::NULL))
    }
}

impl Var<NetF> {
    pub fn get_store(&self) -> &NetStore {
        match self {
            Var::Bound(store) => store,
            Var::Free(store) => store,
        }
    }
}

#[derive(Debug)]
pub struct Net<'a> {
    symbols: &'a SymbolBook,
    pub head: Vec<VarPtr>,
    pub body: Equations<NetF>,
    pub heap: Heap<NetF>,
}
impl<'a> Net<'a> {
    pub fn new(symbols: &'a SymbolBook) -> Self {
        Net::with_capacity(symbols, [0, 0, 0])
    }

    pub fn with_capacity(symbols: &'a SymbolBook, capacity: [usize; 3]) -> Self {
        Self {
            symbols,
            head: Vec::new(),
            body: Equations::with_capacity(capacity[0]),
            heap: Heap::with_capacity(capacity[1], capacity[2]),
        }
    }

    // Equations --------------------------

    pub fn equations<F>(&mut self, builder_fn: F)
    where
        F: FnOnce(&mut EquationBuilder<NetF>),
    {
        let mut builder = EquationBuilder::new(
            &self.symbols,
            &mut self.head,
            &mut self.body,
            &mut self.heap,
        );
        builder_fn(&mut builder);
        builder.build();
    }

    pub fn body(&self) -> ArenaPtrIter<Equation<NetF>, EquationPtr> {
        self.body.iter()
    }

    pub fn get_body(&'a self, equation: EquationPtr) -> &'a Equation<NetF> {
        self.body.get(equation).unwrap()
    }

    // Cells --------------------------

    pub fn cells(&self) -> ArenaPtrIter<Cell<NetF>, CellPtr> {
        self.heap.cells()
    }

    pub fn get_cell(&'a self, cell: CellPtr) -> &'a Cell<NetF> {
        self.heap.get_cell(cell).unwrap()
    }

    // Vars --------------------------

    pub fn vars(&self) -> ArenaPtrIter<Var<NetF>, VarPtr> {
        self.heap.vars()
    }

    pub fn get_var(&'a self, ptr: VarPtr) -> &'a Var<NetF> {
        &self.heap.get_var(ptr).unwrap()
    }

    pub fn fvar(&mut self) -> VarPtr {
        let ptr = self.heap.fvar(NetStore::default());
        self.head.push(ptr);
        ptr
    }

    pub fn bvar(&mut self) -> VarPtr {
        self.heap.bvar(NetStore::default())
    }

    pub fn display_head(&'a self) -> HeadDisplay {
        HeadDisplay { net: self }
    }

    pub fn display_body(&'a self) -> EquationsDisplay<'a, NetF> {
        EquationsDisplay {
            symbols: self.symbols,
            body: &self.body,
            heap: &self.heap,
        }
    }

    pub fn display_equation(
        &'a self,
        symbols: &'a SymbolBook,
        equation: &'a Equation<NetF>,
    ) -> EquationDisplay<'a, NetF> {
        EquationDisplay {
            equation,
            symbols,
            heap: &self.heap,
        }
    }

    pub fn display_cell( &'a self, cell_ptr: CellPtr) -> CellDisplay<'a, NetF> {
        self.heap.display_cell(self.symbols, cell_ptr)
    }
}

impl<'a> Display for Net<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} | {} >", self.display_head(), self.display_body())
    }
}

pub struct HeadDisplay<'a> {
    net: &'a Net<'a>,
}
impl<'a> Display for HeadDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.net.head.iter().fold(Ok(()), |result, fvar_ptr| {
            result.and_then(|_| {
                let fvar = self.net.get_var(*fvar_ptr);
                assert!(fvar.is_free());
                match fvar {
                    Var::Bound(_) => unreachable!(),
                    Var::Free(store) => match store.get_cell_ptr() {
                        Some(cell_ptr) => write!(
                            f,
                            " _.{}={}",
                            fvar_ptr.get_index(),
                            self.net.heap.display_cell(self.net.symbols, cell_ptr)
                        ),
                        None => write!(f, " _.{}", fvar_ptr.get_index()),
                    },
                }
            })
        })
    }
}

pub trait NetVisitor<T: TermFamily> {
    fn visit_redex(&mut self, eqn_ptr: EquationPtr, ctr: CellPtr, fun: CellPtr) -> bool {
        true
    }

    fn visit_bind(&mut self, eqn_ptr: EquationPtr, var: VarPtr, cell: CellPtr) -> bool {
        true
    }

    fn visit_connect(&mut self, eqn_ptr: EquationPtr, left: VarPtr, right: VarPtr) -> bool {
        true
    }

    fn visit_cell0(&mut self, cell_ptr: CellPtr, sym_ptr: SymbolPtr) {}

    fn visit_cell1(&mut self, cell_ptr: CellPtr, sym_ptr: SymbolPtr, port: TermPtr) -> bool {
        true
    }

    fn visit_cell2(
        &mut self,
        cell_ptr: CellPtr,
        sym_ptr: SymbolPtr,
        left: TermPtr,
        right: TermPtr,
    ) -> bool {
        true
    }

    fn visit_var(&mut self, var_ptr: VarPtr, fvar: &Var<T>) {}
}
