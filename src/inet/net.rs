
use std::fmt::Display;

use crate::inet::equation::EquationKind;

use super::{equation::{Equation, EquationPtr, Equations}, cell::{Cell, CellPtr, PortPtr, Cells, PortKind}, var::{VarPtr, FVar, BVar, FVarPtr, BVarPtr, FVars, BVars}, symbol::{SymbolPtr, SymbolBook, SymbolArity}, arena::ArenaIter, Polarity};

#[derive(Debug)]
pub struct Net<'a> {
    equations : Equations,
    cells: Cells,
    bvars: BVars,
    fvars: FVars,
    symbols: &'a SymbolBook
}
impl<'a> Net<'a> {
    pub fn new(symbols: &'a SymbolBook) -> Self {
        Self {
            equations: Equations::new(),
            cells: Cells::new(),
            bvars: BVars::new(),
            fvars: FVars::new(),
            symbols
        }
    }

    pub fn with_capacity(symbols: &'a SymbolBook, capacity: [usize;4]) -> Self {
        Self {
            equations: Equations::with_capacity(capacity[0]),
            cells: Cells::with_capacity(capacity[1]),
            bvars: BVars::with_capacity(capacity[2]),
            fvars: FVars::with_capacity(capacity[3]),
            symbols
        }
    }

    // Equations --------------------------

    pub fn redex(&mut self, ctr: CellPtr, fun: CellPtr) -> EquationPtr {
        self.equations.add(Equation::redex(ctr, fun))
    }

    pub fn bind(&mut self, var: VarPtr, cell: CellPtr) -> EquationPtr {
        self.equations.add(Equation::bind(var, cell))
    }

    pub fn connect(&mut self, left: VarPtr, right: VarPtr) -> EquationPtr {
        self.equations.add(Equation::connect(left, right))
    }

    pub fn equations(&self) -> ArenaIter<Equation,EquationPtr> {
        self.equations.iter()
    }

    pub fn get_equation(&self, equation: EquationPtr) -> Equation {
        self.equations.get(equation)
    }

    // Cells --------------------------

    pub fn cell0(&mut self, symbol: SymbolPtr) -> CellPtr {
        self.cells.add(Cell::new0(symbol))
    }

    pub fn cell1(&mut self, symbol: SymbolPtr, left_port: PortPtr) -> CellPtr {
        self.cells.add(Cell::new1(symbol, left_port))
    }

    pub fn cell2(&mut self, symbol: SymbolPtr, left_port: PortPtr, right_port: PortPtr) -> CellPtr {
        self.cells.add(Cell::new2(symbol, left_port, right_port))
    }

    pub fn cells(&self) -> ArenaIter<Cell,CellPtr> {
        self.cells.iter()
    }

    pub fn get_cell(&self, cell: CellPtr) -> Cell {
        self.cells.get(cell)
    }

    // FVars --------------------------


    pub fn fvar(&mut self) -> FVarPtr {
        self.fvars.add(FVar::default())
    }

    pub fn fvars(&self) -> ArenaIter<FVar,FVarPtr> {
        self.fvars.iter()
    }

    pub fn get_fvar(&self, ptr: FVarPtr) -> &FVar {
        self.fvars.get(ptr)
    }

    // BVars ------

    pub fn bvar(&mut self) -> BVarPtr {
        self.bvars.add(BVar::default())
    }

    pub fn bvars(&self) -> ArenaIter<BVar,BVarPtr> {
        self.bvars.iter()
    }

    pub fn display_cell(&self, cell_ptr: CellPtr) -> CellItem {
        CellItem(cell_ptr, &self)
    }
}


impl<'a> Display for Net<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "< {} | {} >", FVarsItem(&self.fvars, self), EquationsItem(&self.equations, self))
    }
}

struct FVarsItem<'a>(&'a FVars, &'a Net<'a>);
impl<'a> Display for FVarsItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().fold(Ok(()), |result, ptr| {
            result.and_then(|_| write!(f, " {}", VarItem(ptr.into(), self.1)))
        })
    }
}


struct EquationsItem<'a>(&'a Equations, &'a Net<'a>);
impl<'a> Display for EquationsItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().fold(Ok(()), |result, eqn_ptr| {
            result.and_then(|_| write!(f, " {}", EquationItem(eqn_ptr, self.1)))
        })
    }
}

struct EquationItem<'a>(EquationPtr, &'a Net<'a>);

impl<'a> EquationItem<'a> {
    pub fn get(&self) -> Equation {
        self.1.get_equation(self.0)
    }
}

impl<'a> Display for EquationItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let eqn = self.get();
        match eqn.get_kind() {
            EquationKind::Redex => {
                write!(f, "{} = {}", CellItem(eqn.get_redex_left(), &self.1), CellItem(eqn.get_redex_right(), &self.1))
            },
            EquationKind::Bind => {
                let cell = CellItem(eqn.get_bind_cell(), self.1);
                let var = VarItem(eqn.get_bind_var(), self.1);
                write!(f, "{} ← {}", var, cell)
            },
            EquationKind::Connect => {
                let left = VarItem(eqn.get_connect_left(), self.1);
                let right = VarItem(eqn.get_connect_right(), self.1);
                write!(f, "{} ← {}", left, right)
            },
        }
    }
}


pub struct CellItem<'a>(CellPtr, &'a Net<'a>);

impl<'a> CellItem<'a> {
    pub fn get(&self) -> Cell {
        self.1.get_cell(self.0)
    }
}

impl<'a> Display for CellItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cell = self.get();

        let name = self.1.symbols.get_name(cell.get_symbol());
        let symbol = self.1.symbols.get(cell.get_symbol());
        match symbol.get_arity() {
            SymbolArity::Zero => {
                write!(f, "{}", name)
            },
            SymbolArity::One => {
                let port = cell.get_left_port();
                write!(f, "({} {})", name,  PortItem(port, self.1, symbol.get_left_polarity()))
            },
            SymbolArity::Two => {
                let left_port = PortItem(cell.get_left_port(), self.1, symbol.get_left_polarity());
                let right_port = PortItem(cell.get_right_port(), self.1, symbol.get_right_polarity());
                write!(f, "({} {} {})", name, left_port, right_port)
            },
        }
    }
}

struct PortItem<'a>(PortPtr, &'a Net<'a>, Polarity);

impl<'a> Display for PortItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.get_kind() {
            PortKind::Cell => {
                CellItem(self.0.into(), &self.1).fmt(f)
            },
            PortKind::FVar => {
                VarItem(self.0.get_fvar().into(), self.1).fmt(f)
            },
            PortKind::BVar => {
                VarItem(self.0.get_bvar().into(), self.1).fmt(f)
            },
        }
    }
}

struct VarItem<'a>(VarPtr, &'a Net<'a>);
impl<'a> Display for VarItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.is_free() {
            true => write!(f, "_.{}", self.0.get_index()),
            false => write!(f, "x.{}", self.0.get_index()),
        }
    }
}