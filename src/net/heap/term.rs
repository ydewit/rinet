
use std::fmt::Debug;
pub use std::sync::Arc;
use std::sync::OnceLock;

use crate::net::{Polarity, Polarized};

use super::symbol::{Symbol, SymbolBook};


pub trait NetFamily {
    type FVar : Debug;
    type BVar : Debug;
}

#[derive(Debug)]
pub enum Var<F: NetFamily = TermFamily>{
    FVar(Arc<F::FVar>),
    BVar(Arc<F::BVar>),
}

pub type BVar<F: NetFamily = TermFamily> = F::BVar;
pub type FVar<F: NetFamily = TermFamily> = F::FVar;

impl Var<TermFamily> {
    pub fn bvar() -> Var {
        Var::BVar(Arc::new(OnceLock::new()))
    }

    pub fn fvar() -> Var {
        Var::BVar(Arc::new(OnceLock::new()))
    }
}

#[derive(Debug)]
pub struct Cell<F: NetFamily = TermFamily> {
    pub symbol: Arc<Symbol>, // we may have multiple cells referencing CellDecl
    pub ports: Vec<Box<Term<F>>>
}

impl<F: NetFamily> Cell<F> {
    pub fn new(symbol: &Arc<Symbol>, ports: Vec<Box<Term<F>>>) -> Self {
        Self { symbol: symbol.clone(), ports: ports }
    }

    pub fn is(&self, symbol: &Arc<Symbol>) -> bool{
        let a = Box::new(1);
        let b = &a;
        self.symbol == *symbol
    }
}
impl<F: NetFamily> Polarized for Cell<F> {
    fn polarity(self: &Self) -> &Polarity {
        &self.symbol.polarity()
    }
}

#[derive(Debug)]
pub enum Term<F: NetFamily = TermFamily> {
    Agent(Box<Cell<F>>),
    Var(Var<F>)
}

impl<F: NetFamily> From<Box<Cell<F>>> for Term<F> {
    fn from(value: Box<Cell<F>>) -> Self {
        Self::Agent(value)
    }
}

impl<F: NetFamily> From<Var<F>> for Box<Term<F>> {
    fn from(value: Var<F>) -> Self {
        Self::new(Term::Var(value))
    }
}

// impl<F: NetFamily> Polarized for Term<F> {
//     fn polarity(self: &Self) -> Polarity {
//         match self {
//             Term::Agent(cell) => Polarity::cell.polarity(),
//             Term::Var(var) => var.polarity()
//         }
//     }
// }

#[derive(Debug)]
pub enum Equation<F: NetFamily = TermFamily> {
    Redex(Box<Cell<F>>, Box<Cell<F>>),
    Bind(Var<F>, Box<Cell<F>>),
    Connect(Var<F>, Var<F>)
}

impl<F: NetFamily> Equation<F> {
    pub fn is_short_circuit(self: &Self) -> bool {
        match self {
            Equation::Redex(lhs, rhs) => lhs.polarity() == rhs.polarity(),
            Equation::Bind(var, cell) => true, //var.polarity() == cell.polarity(),
            Equation::Connect(lhs, rhs) => true //lhs.polarity() == rhs.polarity(),
        }
    }
}


#[derive(Debug)]
pub struct Net {
    pub head: Vec<Arc<FVar>>,
    pub body: Vec<Equation>
}

impl Net {
    pub fn new(symbols: &Arc<SymbolBook>, action: fn(&mut NetBuilder) -> ()) -> Self {
        let mut builder = NetBuilder::new(symbols);
        action(&mut builder);
        builder.build()
    }
}

pub struct NetBuilder<'a> {
    symbols: &'a Arc<SymbolBook>,
    head: Vec<Arc<FVar>>,
    equations: Vec<Equation>,
    // bvars: u8
}

impl<'a> NetBuilder<'a> {
    fn new(symbols: &'a Arc<SymbolBook>) -> Self {
        Self { symbols: symbols, head: Vec::new(), equations: Vec::new() }
    }

    pub fn fresh(&mut self) -> Var {
        Var::bvar()
    }

    pub fn fvar(&mut self) -> Var {
        Var::fvar()
    }

    pub fn cell(&self, name: &str, ports: Vec<Box<Term>>) -> Box<Cell> {
        let symbol = self.symbols.find(name).unwrap();
        let cell = Cell::new(symbol, ports);
        Box::new(cell)
    }

    pub fn redex(&mut self, lhs: Box<Cell>, rhs: Box<Cell>) -> &Self {
        let eqn = Equation::Redex(lhs, rhs);
        self.equations.push(eqn);
        self
    }

    pub fn bind(&mut self, var: Var, rhs: Box<Cell>) -> &Self {
        let eqn = Equation::Bind(var, rhs);
        self.equations.push(eqn);
        self
    }

    fn build(self) -> Net {
        Net { head: self.head, body: self.equations }
    }
}

#[derive(Debug)]
pub struct TermFamily {}

impl NetFamily for TermFamily {
    type FVar = OnceLock<Box<Cell>>;
    type BVar = OnceLock<Box<Cell>>;
}
