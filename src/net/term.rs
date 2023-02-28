use std::{sync::OnceLock, fmt::Debug};

use super::{arena::{Ptr, Arena, ArenaIter, ToTag}, symbol::{SymbolTag, self}, dsl::{TermDsl, EquationDsl}, Polarity};

/// ---------------------------------------------
/// ## Expr
/// ---------------------------------------------
///
#[derive(Debug,Clone,Copy)]
pub enum CellTag {
    Cell0(Ptr<SymbolTag>),
    Cell1(Ptr<SymbolTag>),
    Cell2(Ptr<SymbolTag>)
}
impl CellTag {
    pub fn get_symbol(&self) -> Ptr<SymbolTag> {
        match self {
            CellTag::Cell0(symbol) => *symbol,
            CellTag::Cell1(symbol) => *symbol,
            CellTag::Cell2(symbol) => *symbol
        }
    }
}
impl Into<Port> for Ptr<CellTag> {
    fn into(self) -> Port {
        Port::Cell(self)
    }
}

#[derive(Debug)]
pub enum Cell {
    Cell0 {
        symbol: Ptr<SymbolTag>
    },
    Cell1 {
        symbol: Ptr<SymbolTag>,
        port: Port
    },
    Cell2 {
        symbol: Ptr<SymbolTag>,
        left_port: Port,
        right_port: Port
    }
}
impl ToTag<CellTag> for Cell {
    fn to_tag(&self) -> CellTag {
        match self {
            Cell::Cell0 { symbol } => CellTag::Cell0(*symbol),
            Cell::Cell1 { symbol, .. } => CellTag::Cell1(*symbol),
            Cell::Cell2 { symbol, .. } => CellTag::Cell1(*symbol),
        }
    }
}


#[derive(Debug,Clone,Copy)]
pub enum VarTag {
    FVar,
    BVar
}

impl Into<Port> for Ptr<VarTag> {
    fn into(self) -> Port {
        Port::Var(self)
    }
}


#[derive(Debug)]
struct FVarEntry<F> {
    store: F
}

impl<F> ToTag<VarTag> for FVarEntry<F> {
    fn to_tag(&self) -> VarTag {
        VarTag::FVar
    }
}

#[derive(Debug)]
struct BVarEntry<B> {
    store: B
}

impl<B> ToTag<VarTag> for BVarEntry<B> {
    fn to_tag(&self) -> VarTag {
        VarTag::BVar
    }
}


#[derive(Debug,Clone, Copy)]
pub enum Port {
    Cell0(Ptr<SymbolTag>),
    Cell(Ptr<CellTag>),
    Var(Ptr<VarTag>),
}

/// ---------------------------------------------
/// ## Equation
/// ---------------------------------------------
#[derive(Debug, Clone, Copy)]
pub enum EquationTag {
    Redex,
    Bind,
    Connect
}

#[derive(Debug, Clone, Copy)]
pub enum Equation {
    Redex{
        ctr: Ptr<CellTag>,
        fun: Ptr<CellTag>
    },
    Bind {
        var: Ptr<VarTag>,
        cell: Ptr<CellTag>
    },
    Connect {
        left: Ptr<VarTag>,
        right: Ptr<VarTag>
    },
}
impl ToTag<EquationTag> for Equation {
    fn to_tag(&self) -> EquationTag {
        match self {
            Equation::Redex { ctr, fun } => EquationTag::Redex,
            Equation::Bind { var, cell } => EquationTag::Bind,
            Equation::Connect { left, right } => EquationTag::Connect,
        }
    }
}

#[derive(Debug)]
pub struct Net<F: Debug = FVar, B: Debug = BVar> {
    equations: Arena<Equation, EquationTag>,
    cells: Arena<Cell, CellTag>,
    fvars: Arena<FVarEntry<F>, VarTag>,
    bvars: Arena<BVarEntry<B>, VarTag>
}

impl<F: Debug, B: Debug> Net<F,B> {
    pub fn equations<'a>(&'a self) -> impl Iterator<Item=Ptr<EquationTag>> {
        self.equations.iter()
    }

    pub fn get_equation(&self, equation: &Ptr<EquationTag>) -> Option<&Equation> {
        self.equations.get(&equation)
    }

    pub fn cells<'a>(&'a self) -> impl Iterator<Item=Ptr<CellTag>> + 'a {
        self.cells.iter()
    }

    pub fn get_cell(&self, ptr: &Ptr<CellTag>) -> Option<&Cell> {
        self.cells.get(&ptr)
    }
}

impl<F: Debug, B: Debug> Default for Net<F, B> {
    fn default() -> Self {
        Self {
            equations: Arena::new(),
            cells: Arena::new(),
            fvars: Arena::new(),
            bvars: Arena::new()
        }
    }
}

impl<F: Debug, B: Debug> EquationDsl for Net<F,B> {
    type EquationRef = Ptr<EquationTag>;
    type CellRef = Ptr<CellTag>;
    type VarRef = Ptr<VarTag>;

    fn redex(&mut self, ctr: Self::CellRef, fun: Self::CellRef) -> Self::EquationRef {
        self.equations.alloc(Equation::Redex { ctr, fun }, EquationTag::Redex)
    }

    fn bind(&mut self, var: Self::VarRef, cell: Self::CellRef) -> Self::EquationRef {
        self.equations.alloc(Equation::Bind { var, cell }, EquationTag::Bind)
    }

    fn connect(&mut self, left: Self::VarRef, right: Self::VarRef) -> Self::EquationRef {
        self.equations.alloc(Equation::Connect { left, right }, EquationTag::Connect)
    }
}

impl<F: Debug, B: Debug> TermDsl for Net<F,B> {
    type SymbolRef = Ptr<SymbolTag>;
    type Port = Port;

    type CellRef = Ptr<CellTag>;

    type VarRef = Ptr<VarTag>;

    type FVar = F;

    type BVar = B;

    fn cell0(&mut self, symbol: Self::SymbolRef) -> Ptr<CellTag> {
        self.cells.alloc(Cell::Cell0 { symbol }, CellTag::Cell0(symbol))
    }

    fn cell1(&mut self, symbol: Self::SymbolRef, port: Self::Port) -> Self::CellRef {
        self.cells.alloc(Cell::Cell1 { symbol, port }, CellTag::Cell1(symbol))
    }

    fn cell2(&mut self, symbol: Self::SymbolRef, left_port: Self::Port, right_port: Self::Port) -> Self::CellRef {
        self.cells.alloc(Cell::Cell2 { symbol, left_port, right_port }, CellTag::Cell2(symbol))
    }

    fn fvar(&mut self, var: Self::FVar) -> Self::VarRef {
        self.fvars.alloc(FVarEntry { store: var }, VarTag::FVar)
    }

    fn bvar(&mut self, var: Self::BVar) -> Self::VarRef {
        self.bvars.alloc(BVarEntry { store: var }, VarTag::BVar)
    }
}


#[derive(Debug)]
pub struct BVar {
    value: OnceLock<Ptr<CellTag>>
}

impl BVar {
    pub fn new() -> Self {
        Self { value: OnceLock::default() }
    }
}
#[derive(Debug)]
pub struct FVar {
    polarity: Polarity,
    value: OnceLock<Ptr<CellTag>>
}

impl FVar {
    pub fn new(polarity: Polarity) -> Self {
        Self { polarity, value: OnceLock::default() }
    }
}