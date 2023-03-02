use std::{
    fmt::Display,
    sync::atomic::{AtomicU32, Ordering},
};

use super::{
    arena::ArenaPtrIter,
    cell::{Cell, CellPtr},
    equation::{Equation, EquationPtr, Equations, EquationsDisplay},
    heap::Heap,
    symbol::{SymbolBook, SymbolPtr},
    term::{TermFamily, TermPtr},
    var::{Var, VarPtr},
};

#[derive(Debug, Copy, Clone)]
pub struct NetF {}
impl TermFamily for NetF {
    type Store = NetStore;

    fn display_store(
        f: &mut std::fmt::Formatter<'_>,
        symbols: &SymbolBook,
        heap: &Heap<Self>,
        store: &Self::Store,
        index: usize,
    ) -> std::fmt::Result {
        match store.get() {
            Some(cell_ptr) => heap.display_cell(symbols, cell_ptr).fmt(f),
            None => write!(f, "_.{}", index),
        }
    }
}

#[derive(Debug)]
pub struct NetStore(AtomicU32);
impl NetStore {
    const NULL: u32 = u32::MAX;

    pub fn get(&self) -> Option<CellPtr> {
        match self.0.load(Ordering::SeqCst) {
            Self::NULL => None,
            ptr => {
                let other_cell_ptr = CellPtr::from(ptr);
                // // only a Pos cell is "the real" value for a var. If
                // // it is a Neg, then it is just a temporary value
                // match other_cell_ptr.get_polarity() {
                //     super::Polarity::Pos =>
                Some(other_cell_ptr)
                //     super::Polarity::Neg => None,
                // }
            }
        }
    }

    pub fn get_or_set(&self, cell_ptr: CellPtr) -> Option<CellPtr> {
        match self.0.swap(cell_ptr.get_ptr(), Ordering::SeqCst) {
            // first time setting
            Self::NULL => None,
            // another cell_ptr already set
            ptr => {
                let other_cell_ptr = CellPtr::from(ptr);
                // if other_cell_ptr.get_polarity() == Polarity::Pos {
                //     // if a Neg cell_ptr bumped the Pos one out of the var, store it back
                //     // The final value of a var should always be a Pos cell_ptr
                //     // Note: there should be at most two calls to get_or_set()
                //     self.0.store(other_cell_ptr.get_ptr(), Ordering::SeqCst);
                // }
                Some(other_cell_ptr)
            }
        }
    }
}

impl Default for NetStore {
    fn default() -> Self {
        Self(AtomicU32::new(Self::NULL))
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

    pub fn var(&mut self) -> VarPtr {
        self.net.var()
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
    pub fn new<F: FnOnce(&mut NetBuilder)>(builderFn: F) -> Self {
        Net::with_capacity([0, 0, 0], builderFn)
    }

    pub fn with_capacity<F: FnOnce(&mut NetBuilder)>(capacity: [usize; 3], builderFn: F) -> Self {
        let mut builder = NetBuilder {
            net: Net {
                head: Vec::new(),
                body: Equations::with_capacity(capacity[0]),
                heap: Heap::with_capacity(capacity[1], capacity[2]),
            },
        };
        builderFn(&mut builder);
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

    pub fn try_set_var(&mut self, var_ptr: VarPtr, cell_ptr: CellPtr) -> Option<CellPtr> {
        match self.heap.get_var(var_ptr).unwrap().0.get_or_set(cell_ptr) {
            Some(other_cell_ptr) => Some(other_cell_ptr),
            None => None,
        }
    }

    pub fn var(&mut self) -> VarPtr {
        let ptr = self.heap.var(NetStore::default());
        self.head.push(ptr);
        ptr
    }

    pub fn display_body<'a>(&'a self, symbols: &'a SymbolBook) -> EquationsDisplay<'a, NetF> {
        EquationsDisplay {
            symbols: symbols,
            body: &self.body,
            heap: &self.heap,
        }
    }

    pub fn display_net<'a>(&'a self, symbols: &'a SymbolBook) -> NetDisplay {
        NetDisplay::new(symbols, self)
    }
}

pub struct NetDisplay<'a> {
    symbols: &'a SymbolBook,
    net: &'a Net,
}

impl<'a> NetDisplay<'a> {
    pub fn new(symbols: &'a SymbolBook, net: &'a Net) -> Self {
        Self { symbols, net }
    }

    fn to_equations_item(&self) -> EquationsDisplay<NetF> {
        EquationsDisplay {
            symbols: self.symbols,
            body: &self.net.body,
            heap: &self.net.heap,
        }
    }

    fn to_head_item(&self) -> HeadVarsItem {
        HeadVarsItem {
            symbols: self.symbols,
            net: self.net,
        }
    }
}

impl<'a> Display for NetDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "< {} | {} >",
            self.to_head_item(),
            self.to_equations_item()
        )
    }
}

pub struct HeadVarsItem<'a> {
    symbols: &'a SymbolBook,
    net: &'a Net,
}
impl<'a> Display for HeadVarsItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.net.head.iter().fold(Ok(()), |result, fvar_ptr| {
            result.and_then(|_| {
                let fvar = self.net.get_var(*fvar_ptr);
                match fvar.0.get() {
                    Some(cell_ptr) => self.net.heap.display_cell(self.symbols, cell_ptr).fmt(f),
                    None => write!(f, "_.{}", fvar_ptr.get_index()),
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
