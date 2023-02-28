
use std::fmt::Display;

use super::{equation::{Equation, EquationPtr, Equations, EquationsItem}, cell::{Cell, CellPtr, PortPtr, Cells, CellItem}, var::{VarPtr, FVar, BVar, FVarPtr, BVarPtr, FVars, BVars, FreeStore, BoundStore, VarKind, VarsItem, VarStore}, symbol::{SymbolPtr, SymbolBook}, arena::ArenaIter, Polarity};

pub struct NetBuilder {
    net: Net
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

    pub fn fvar(&mut self) -> FVarPtr {
        self.net.fvars.add(FVar::default())
    }

    // -------------------

    pub fn bvar(&mut self) -> BVarPtr {
        self.net.bvars.add(BVar::default())
    }

    fn build(self) -> Net {
        self.net
    }
}

#[derive(Debug)]
pub struct Net {
    pub equations : Equations,
    pub cells: Cells,
    pub bvars: BVars,
    pub fvars: FVars,
    // symbols: &'a SymbolBook
}
impl Net {
    pub fn new<F: FnOnce(&mut NetBuilder)>(builderFn: F) -> Self {
        Net::with_capacity([0,0,0,0], builderFn)
    }

    pub fn with_capacity<F: FnOnce(&mut NetBuilder)>(capacity: [usize;4], builderFn: F) -> Self {
        let mut builder = NetBuilder { net: Net {
            equations: Equations::with_capacity(capacity[0]),
            cells: Cells::with_capacity(capacity[1]),
            bvars: BVars::with_capacity(capacity[2]),
            fvars: FVars::with_capacity(capacity[3]),
            // symbols
        } };
        builderFn(&mut builder);
        builder.build()
    }

    // Equations --------------------------

    pub fn equations(&self) -> ArenaIter<Equation,EquationPtr> {
        self.equations.iter()
    }

    pub fn get_equation(&self, equation: EquationPtr) -> &Equation {
        self.equations.get(equation)
    }

    // Cells --------------------------


    pub fn cells(&self) -> ArenaIter<Cell,CellPtr> {
        self.cells.iter()
    }

    pub fn get_cell(&self, cell: CellPtr) -> &Cell {
        &self.cells.get(cell)
    }

    // FVars --------------------------


    // pub fn fvar(&mut self) -> FVarPtr {
    //     self.fvars.add(FVar::default())
    // }

    pub fn fvars(&self) -> ArenaIter<FVar,FVarPtr> {
        self.fvars.iter()
    }

    pub fn get_fvar(&self, ptr: FVarPtr) -> &FVar {
        self.fvars.get(ptr)
    }

    // BVars ------

    // pub fn bvar(&mut self) -> BVarPtr {
    //     self.bvars.add(BVar::default())
    // }

    // pub fn alloc_bvars(&mut self, count: usize) ->  &[Option<BVarPtr>]{
    //     // TODO allocate new bvars in the stack!
    //     let bvars = SmallVector::<Option<BVarPtr>, 10>::new(None, count);
    //     for i in bvars.as_mut_slice() {
    //         let var =  self.bvar();
    //         *i = Some(var);
    //     }
    //     bvars.as_slice()
    // }

    pub fn bvars(&self) -> ArenaIter<BVar,BVarPtr> {
        self.bvars.iter()
    }

    pub fn get_bvar(&self, ptr: BVarPtr) -> &BVar {
        self.bvars.get(ptr)
    }

    // ------------------------

    // pub(crate) fn try_set_var(&self, var_ptr: VarPtr, cell_ptr: CellPtr) -> Option<CellPtr> {
    //     assert!(cell_ptr.get_polarity() == Polarity::Pos);
    //     match var_ptr.get_kind() {
    //         VarKind::Bound => {
    //             let bvar = self.bvars.get(var_ptr.into());
    //             bvar.get_store().try_set(cell_ptr)
    //         }
    //         VarKind::Free => {
    //             let fvar = self.fvars.get(var_ptr.into());
    //             fvar.get_store().send(cell_ptr);
    //             None
    //         },
    //     }
    // }

    // pub(crate) fn try(&self, var_ptr: VarPtr) -> Option<CellPtr> {
    //     match var_ptr.get_kind() {
    //         VarKind::Bound => {
    //             let bvar = self.bvars.get(var_ptr.into());
    //             bvar.get_store().try_get()
    //         },
    //         VarKind::Free => {
    //             let fvar = self.fvars.get(var_ptr.into());
    //             fvar.get_store().try_receive()
    //         }
    //     }
    // }

    // pub fn walk<V: NetVisitor>(&self, visitor: V) {
    //     for eqn_ptr in self.equations() {
    //         self.walk_equation(visitor, eqn_ptr, &self.get_equation(eqn_ptr));
    //     }
    // }

    // pub fn walk_equation<V: NetVisitor>(&self, visitor: V, eqn_ptr: EquationPtr, eqn: &Equation) {
    //     match eqn.get_kind() {
    //         EquationKind::Redex => {
    //             let ctr_ptr = eqn.get_redex_ctr();
    //             let fun_ptr = eqn.get_redex_fun();
    //             if visitor.visit_redex(eqn_ptr, ctr_ptr, fun_ptr) {
    //                 self.walk_cell(visitor, ctr_ptr);
    //                 self.walk_cell(visitor, fun_ptr);
    //             }
    //         },
    //         EquationKind::Bind => {
    //             let var_ptr = eqn.get_bind_var();
    //             let cell_ptr = eqn.get_bind_cell();
    //             if visitor.visit_bind(eqn_ptr, var_ptr, cell_ptr) {
    //                 self.walk_var(visitor, var_ptr);
    //                 self.walk_cell(visitor, cell_ptr);
    //             }
    //         },
    //         EquationKind::Connect => {
    //             let left_ptr = eqn.get_connect_left();
    //             let right_ptr = eqn.get_connect_right();
    //             if visitor.visit_connect(eqn_ptr, left_ptr, right_ptr) {
    //                 self.walk_var(visitor, left_ptr);
    //                 self.walk_var(visitor, right_ptr);
    //             }
    //         },
    //     }
    // }

    // fn walk_cell<V: NetVisitor>(&self, visitor: V, cell_ptr: CellPtr) {
    //     let cell = self.get_cell(cell_ptr);
    //     match cell.get_symbol().get_arity() {
    //         SymbolArity::Zero => visitor.visit_cell0(cell_ptr, cell.get_symbol()),
    //         SymbolArity::One => {
    //             let port_ptr = cell.get_left_port();
    //             if visitor.visit_cell1(cell_ptr, cell.get_symbol(), port_ptr) {
    //                 self.walk_port(visitor, port_ptr);
    //             }
    //         },
    //         SymbolArity::Two => {
    //             let left_ptr = cell.get_left_port();
    //             let right_ptr = cell.get_right_port();
    //             if visitor.visit_cell2(cell_ptr, cell.get_symbol(), left_ptr, right_ptr) {
    //                 self.walk_port(visitor, left_ptr);
    //                 self.walk_port(visitor, right_ptr);
    //             }
    //         },
    //     }
    // }

    // fn walk_port<V: NetVisitor>(&self, visitor: V, port_ptr: PortPtr) {
    //     match port_ptr.get_kind() {
    //         PortKind::Cell => self.walk_cell(visitor, port_ptr.get_cell()),
    //         PortKind::Var => self.walk_var(visitor, port_ptr.get_var())
    //     }
    // }

    // fn walk_var<V: NetVisitor>(&self, visitor: V, var_ptr: VarPtr) {
    //     match var_ptr.is_free() {
    //         true => self.walk_fvar(visitor, var_ptr.into()),
    //         false => self.walk_bvar(visitor, var_ptr.into()),
    //     }
    // }

    // fn walk_fvar<V: NetVisitor>(&self, visitor: V, var_ptr: FVarPtr) {
    //     visitor.visit_fvar(var_ptr, self.get_fvar(var_ptr));
    // }

    // fn walk_bvar<V: NetVisitor>(&self, visitor: V, var_ptr: BVarPtr) {
    //     visitor.visit_bvar(var_ptr, self.get_bvar(var_ptr));
    // }
}

impl Net {
    pub fn display_cell<'a>(&'a self, symbols: &'a SymbolBook, cell_ptr: CellPtr) -> CellItem {
        CellItem {
            cell_ptr,
            symbols: symbols,
            cells: &self.cells,
            bvars: &self.bvars,
            fvars: &self.fvars,
        }
    }
}
pub struct NetItem<'a> {
    symbols: &'a SymbolBook,
    net: &'a Net
}
impl<'a> NetItem<'a> {
    pub fn new(symbols: &'a SymbolBook, net: &'a Net) -> Self {
        Self {
            symbols,
            net
        }
    }

    pub fn to_cell_item(&self, cell_ptr: CellPtr) -> CellItem {
        CellItem {
            cell_ptr,
            symbols: self.symbols,
            cells: &self.net.cells,
            bvars: &self.net.bvars,
            fvars: &self.net.fvars,
        }
    }

    fn to_vars_item(&self, kind: VarKind) -> VarsItem {
        VarsItem { kind, fvars: &self.net.fvars, bvars: &self.net.bvars }
    }

    fn to_equations_item(&self) -> EquationsItem {
        EquationsItem {
            symbols: self.symbols,
            equations: &self.net.equations,
            cells: &self.net.cells,
            bvars: &self.net.bvars,
            fvars: &self.net.fvars,
        }
    }
}
impl<'a> Display for NetItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "< {} | {} >", self.to_vars_item(VarKind::Free), self.to_equations_item())
    }
}


pub trait NetVisitor<F: VarStore = FreeStore,B: VarStore = BoundStore> {
    fn visit_redex(&mut self, eqn_ptr: EquationPtr, ctr: CellPtr, fun: CellPtr) -> bool {
        true
    }

    fn visit_bind(&mut self, eqn_ptr: EquationPtr, var: VarPtr, cell: CellPtr) -> bool {
        true
    }

    fn visit_connect(&mut self, eqn_ptr: EquationPtr, left: VarPtr, right: VarPtr) -> bool {
        true
    }

    fn visit_cell0(&mut self, cell_ptr: CellPtr, sym_ptr: SymbolPtr) {
    }

    fn visit_cell1(&mut self, cell_ptr: CellPtr, sym_ptr: SymbolPtr, port: PortPtr) -> bool {
        true
    }

    fn visit_cell2(&mut self, cell_ptr: CellPtr, sym_ptr: SymbolPtr, left: PortPtr, right: PortPtr) -> bool {
        true
    }

    fn visit_fvar(&mut self, var_ptr: FVarPtr, fvar: &FVar<F>) {
    }

    fn visit_bvar(&mut self, var_ptr: BVarPtr, bvar: &BVar<B>) {
    }
}
