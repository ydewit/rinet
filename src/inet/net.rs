use std::{
    fmt::Display,
    sync::atomic::{AtomicU32, Ordering},
};

use super::{
    arena::ArenaIter,
    cell::{Cell, CellItem, CellPtr, Cells, PortItem, PortPtr},
    equation::{Equation, EquationPtr, Equations, EquationsItem},
    symbol::{SymbolBook, SymbolPtr},
    term::TermFamily,
    var::{Var, VarItem, VarPtr, Vars, VarsItem},
};
#[derive(Debug, Copy, Clone)]
pub struct NetF {}
impl TermFamily for NetF {
    type Store = NetStore;

    fn display_store(
        f: &mut std::fmt::Formatter<'_>,
        _: &Self::Store,
        index: usize,
    ) -> std::fmt::Result {
        write!(f, "x.{}", index)
    }
}

#[derive(Debug)]
pub struct NetStore(AtomicU32);
impl NetStore {
    const NULL: u32 = u32::MAX;

    pub fn get(&self) -> Option<CellPtr> {
        match self.0.load(Ordering::SeqCst) {
            Self::NULL => None,
            ptr => Some(CellPtr::from(ptr)),
        }
    }

    pub fn get_or_set(&self, cell_ptr: CellPtr) -> Option<CellPtr> {
        match self.0.swap(cell_ptr.get_ptr(), Ordering::SeqCst) {
            Self::NULL => None,
            ptr => Some(CellPtr::from(ptr)),
        }
    }
}

impl Default for NetStore {
    fn default() -> Self {
        Self(AtomicU32::new(Self::NULL))
    }
}

impl Vars<NetF> {
    pub fn var(&mut self) -> VarPtr {
        self.add(Var::new(NetStore::default()))
    }

    pub fn try_set(&mut self, var_ptr: VarPtr, cell_ptr: CellPtr) -> Option<CellPtr> {
        match self.get(var_ptr).0.get_or_set(cell_ptr) {
            Some(other_cell_ptr) => Some(other_cell_ptr),
            None => None,
        }
    }
}

pub struct NetBuilder {
    net: Net,
}
impl NetBuilder {
    pub fn redex(&mut self, ctr: CellPtr, fun: CellPtr) -> EquationPtr {
        self.net.equations.redex(ctr, fun)
    }

    pub fn reuse_redex(&mut self, ptr: EquationPtr, ctr: CellPtr, fun: CellPtr) {
        self.net.equations.reuse_redex(ptr, ctr, fun)
    }

    pub fn bind(&mut self, var: VarPtr, cell: CellPtr) -> EquationPtr {
        self.net.equations.bind(var, cell)
    }

    pub fn connect(&mut self, left: VarPtr, right: VarPtr) -> EquationPtr {
        self.net.equations.connect(left, right)
    }

    // ----------------

    pub fn cell0(&mut self, symbol: SymbolPtr) -> CellPtr {
        self.net.cells.cell0(symbol)
    }

    pub fn cell1(&mut self, symbol: SymbolPtr, left_port: PortPtr) -> CellPtr {
        self.net.cells.cell1(symbol, left_port)
    }

    pub fn cell2(&mut self, symbol: SymbolPtr, left_port: PortPtr, right_port: PortPtr) -> CellPtr {
        self.net.cells.cell2(symbol, left_port, right_port)
    }

    // -------------------

    pub fn var(&mut self) -> VarPtr {
        self.net.vars.add(Var::new(NetStore::default()))
    }

    // -------------------

    fn build(self) -> Net {
        self.net
    }
}

#[derive(Debug)]
pub struct Net {
    pub head: Vec<VarPtr>,
    pub equations: Equations<NetF>,
    pub cells: Cells<NetF>,
    pub vars: Vars<NetF>,
}
impl Net {
    pub fn new<F: FnOnce(&mut NetBuilder)>(builderFn: F) -> Self {
        Net::with_capacity([0, 0, 0], builderFn)
    }

    pub fn with_capacity<F: FnOnce(&mut NetBuilder)>(capacity: [usize; 3], builderFn: F) -> Self {
        let mut builder = NetBuilder {
            net: Net {
                head: Vec::new(),
                equations: Equations::with_capacity(capacity[0]),
                cells: Cells::with_capacity(capacity[1]),
                vars: Vars::with_capacity(capacity[2]),
                // symbols
            },
        };
        builderFn(&mut builder);
        builder.build()
    }

    // Equations --------------------------

    pub fn equations(&self) -> ArenaIter<Equation<NetF>, EquationPtr> {
        self.equations.iter()
    }

    pub fn get_equation(&self, equation: EquationPtr) -> Equation<NetF> {
        self.equations.get(equation)
    }

    // Cells --------------------------

    pub fn cells(&self) -> ArenaIter<Cell<NetF>, CellPtr> {
        self.cells.iter()
    }

    pub fn get_cell(&self, cell: CellPtr) -> Cell<NetF> {
        self.cells.get(cell)
    }

    // FVars --------------------------

    pub fn vars(&self) -> ArenaIter<Var<NetF>, VarPtr> {
        self.vars.iter()
    }

    pub fn get_fvar(&self, ptr: VarPtr) -> &Var<NetF> {
        &self.vars.get(ptr)
    }

    pub fn display_cell<'a>(
        &'a self,
        symbols: &'a SymbolBook,
        cell_ptr: CellPtr,
    ) -> CellItem<NetF> {
        CellItem {
            cell_ptr,
            symbols: symbols,
            cells: &self.cells,
            vars: &self.vars,
        }
    }

    pub fn display_port<'a>(
        &'a self,
        symbols: &'a SymbolBook,
        port_ptr: PortPtr,
    ) -> PortItem<NetF> {
        PortItem {
            port_ptr,
            symbols: symbols,
            cells: &self.cells,
            vars: &self.vars,
        }
    }

    pub fn display_var<'a>(&'a self, symbols: &'a SymbolBook, var_ptr: VarPtr) -> VarItem<NetF> {
        VarItem {
            var_ptr,
            vars: &self.vars,
        }
    }
}

pub struct NetItem<'a> {
    symbols: &'a SymbolBook,
    net: &'a Net,
}

impl<'a> NetItem<'a> {
    pub fn new(symbols: &'a SymbolBook, net: &'a Net) -> Self {
        Self { symbols, net }
    }

    pub fn to_cell_item(&self, cell_ptr: CellPtr) -> CellItem<NetF> {
        CellItem {
            cell_ptr,
            symbols: self.symbols,
            cells: &self.net.cells,
            vars: &self.net.vars,
        }
    }

    fn to_vars_item(&self) -> VarsItem<NetF> {
        VarsItem {
            vars: &self.net.vars,
        }
    }

    fn to_equations_item(&self) -> EquationsItem<NetF> {
        EquationsItem {
            symbols: self.symbols,
            equations: &self.net.equations,
            cells: &self.net.cells,
            vars: &self.net.vars,
        }
    }
}

impl<'a> Display for NetItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "< {} | {} >",
            self.to_vars_item(),
            self.to_equations_item()
        )
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

    fn visit_cell1(&mut self, cell_ptr: CellPtr, sym_ptr: SymbolPtr, port: PortPtr) -> bool {
        true
    }

    fn visit_cell2(
        &mut self,
        cell_ptr: CellPtr,
        sym_ptr: SymbolPtr,
        left: PortPtr,
        right: PortPtr,
    ) -> bool {
        true
    }

    fn visit_var(&mut self, var_ptr: VarPtr, fvar: &Var<T>) {}
}
