use std::{
    fmt::Display,
    sync::atomic::{AtomicU32, Ordering},
};

use super::{
    arena::ArenaPtrIter,
    cell::{Cell, CellPtr},
    equation::{Equation, EquationPtr, Equations, EquationsDisplay, EquationDisplay},
    heap::Heap,
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
            Var::Bound(store) => {
                match store.get_cell_ptr() {
                    Some(cell_ptr) => {
                        heap.display_cell(symbols, cell_ptr).fmt(f)
                    },
                    None => write!(f, "x{}", index)
                }

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
        }
        else {
            return None
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
        }
        else {
            return None
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

pub struct NetBuilder {
    net: Net,
}
impl NetBuilder {
    pub fn redex(&mut self, ctr_ptr: CellPtr, fun_ptr: CellPtr) -> EquationPtr {
        self.net.redex(ctr_ptr, fun_ptr)
    }

    pub fn bind(&mut self, var_ptr: VarPtr, cell_ptr: CellPtr) -> EquationPtr {
        self.net.bind(var_ptr, cell_ptr)
    }

    pub fn connect(&mut self, left_ptr: VarPtr, right_ptr: VarPtr) -> EquationPtr {
        self.net.connect(left_ptr, right_ptr)
    }

    // ----------------

    pub fn cell0(&mut self, symbol_ptr: SymbolPtr) -> CellPtr {
        self.net.heap.cell0(symbol_ptr)
    }

    pub fn cell1(&mut self, symbol_ptr: SymbolPtr, left_port: TermPtr) -> CellPtr {
        self.net.heap.cell1(symbol_ptr, left_port)
    }

    pub fn cell2(
        &mut self,
        symbol_ptr: SymbolPtr,
        left_port: TermPtr,
        right_port: TermPtr,
    ) -> CellPtr {
        self.net.heap.cell2(symbol_ptr, left_port, right_port)
    }

    // -------------------

    pub fn fvar(&mut self) -> VarPtr {
        self.net.fvar()
    }

    pub fn bvar(&mut self) -> VarPtr {
        self.net.bvar()
    }

    // -------------------

    fn build(self) -> Net {
        self.net
    }
}

#[derive(Debug)]
pub struct Net {
    pub head: Vec<VarPtr>,
    pub body: Equations<NetF>,
    pub heap: Heap<NetF>,
}
impl Net {
    pub fn new<F: FnOnce(&mut NetBuilder)>(builder_fn: F) -> Self {
        Net::with_capacity([0, 0, 0], builder_fn)
    }

    pub fn with_capacity<F: FnOnce(&mut NetBuilder)>(capacity: [usize; 3], builder_fn: F) -> Self {
        let mut builder = NetBuilder {
            net: Net {
                head: Vec::new(),
                body: Equations::with_capacity(capacity[0]),
                heap: Heap::with_capacity(capacity[1], capacity[2]),
            },
        };
        builder_fn(&mut builder);
        builder.build()
    }

    // Equations --------------------------

    pub fn redex(&mut self, ctr: CellPtr, fun: CellPtr) -> EquationPtr {
        self.body.alloc(Equation::redex(ctr, fun))
    }

    pub fn reuse_redex(&mut self, ptr: EquationPtr, ctr: CellPtr, fun: CellPtr) {
        self.body.update(ptr, Equation::redex(ctr, fun));
    }

    pub fn bind(&mut self, var: VarPtr, cell: CellPtr) -> EquationPtr {
        self.body.alloc(Equation::bind(var, cell))
    }

    pub fn reuse_bind(&mut self, ptr: EquationPtr, var_ptr: VarPtr, cell_ptr: CellPtr) {
        self.body.update(ptr, Equation::bind(var_ptr, cell_ptr));
    }

    pub fn connect(&mut self, left: VarPtr, right: VarPtr) -> EquationPtr {
        self.body.alloc(Equation::connect(left, right))
    }

    pub fn reuse_connect(&mut self, ptr: EquationPtr, left_ptr: VarPtr, right_ptr: VarPtr) {
        self.body
            .update(ptr, Equation::connect(left_ptr, right_ptr));
    }

    pub fn body(&self) -> ArenaPtrIter<Equation<NetF>, EquationPtr> {
        self.body.iter()
    }

    pub fn get_body<'a>(&'a self, equation: EquationPtr) -> &'a Equation<NetF> {
        self.body.get(equation).unwrap()
    }

    // Cells --------------------------

    pub fn cells(&self) -> ArenaPtrIter<Cell<NetF>, CellPtr> {
        self.heap.cells()
    }

    pub fn get_cell<'a>(&'a self, cell: CellPtr) -> &'a Cell<NetF> {
        self.heap.get_cell(cell).unwrap()
    }

    // Vars --------------------------

    pub fn vars(&self) -> ArenaPtrIter<Var<NetF>, VarPtr> {
        self.heap.vars()
    }

    pub fn get_var<'a>(&'a self, ptr: VarPtr) -> &'a Var<NetF> {
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

    pub fn display_head<'a>(&'a self, symbols: &'a SymbolBook) -> HeadDisplay {
        HeadDisplay { symbols, net: self }
    }

    pub fn display_body<'a>(&'a self, symbols: &'a SymbolBook) -> EquationsDisplay<'a, NetF> {
        EquationsDisplay {
            symbols: symbols,
            body: &self.body,
            heap: &self.heap,
        }
    }

    pub fn display_equation<'a>(&'a self, symbols: &'a SymbolBook, equation: &'a Equation<NetF>) -> EquationDisplay<'a, NetF> {
        EquationDisplay {
            equation,
            symbols,
            heap: &self.heap,
        }
    }
    pub fn display_net<'a>(&'a self, symbols: &'a SymbolBook) -> NetDisplay {
        NetDisplay { symbols, net: self }
    }
}

pub struct NetDisplay<'a> {
    symbols: &'a SymbolBook,
    net: &'a Net,
}

impl<'a> Display for NetDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "< {} | {} >",
            self.net.display_head(self.symbols),
            self.net.display_body(self.symbols)
        )
    }
}

pub struct HeadDisplay<'a> {
    symbols: &'a SymbolBook,
    net: &'a Net,
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
                        Some(cell_ptr) => self.net.heap.display_cell(self.symbols, cell_ptr).fmt(f),
                        None => write!(f, "_.{}", fvar_ptr.get_index()),
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
