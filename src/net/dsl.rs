use std::{fmt::Debug, slice::Iter};

use super::{Polarity};


/// ## SymbolDsl
///
pub trait SymbolDsl : Default + Debug {
    type CtrSymbolRef : Copy;
    type FunSymbolRef : Copy;

    fn ctr0(&mut self, name: &str) -> Self::CtrSymbolRef;
    fn ctr1(&mut self, name: &str, port: Polarity) -> Self::CtrSymbolRef;
    fn ctr2(&mut self, name: &str, left_port: Polarity, right_port: Polarity) -> Self::CtrSymbolRef;

    fn fun0(&mut self, name: &str) -> Self::FunSymbolRef;
    fn fun1(&mut self, name: &str, port: Polarity) -> Self::FunSymbolRef;
    fn fun2(&mut self, name: &str, left_port: Polarity, right_port: Polarity) -> Self::FunSymbolRef;

}

/// ## SymbolBookDsl
///

pub trait SymbolBookDsl {
    type CtrSymbolRef : Copy;
    type FunSymbolRef : Copy;

    fn find_ctr(&self, name: &str) -> Option<Self::CtrSymbolRef>;

    fn find_fun(&self, name: &str) -> Option<Self::FunSymbolRef>;
}


/// ## TermDsl
///

pub trait TermDsl : Debug {
    type SymbolRef: Copy;
    type Port;
    type CellRef : Copy;
    type VarRef : Copy;
    type FVar;
    type BVar;

    fn cell0(&mut self, symbol: Self::SymbolRef) -> Self::CellRef;

    fn cell1(&mut self, symbol: Self::SymbolRef, port: Self::Port) -> Self::CellRef;

    fn cell2(&mut self, symbol: Self::SymbolRef, left_port: Self::Port, right_port: Self::Port) -> Self::CellRef;

    /// A free variable that serves as the head (interface) of the net and
    /// that has a specific polarity: it is either an input or an output.
    ///
    /// Note that a Net is linear and a free-variable has at least one end and
    /// at most two, in case the Net is composed with another Net.
    fn fvar(&mut self, var: Self::FVar) -> Self::VarRef;

    /// A bound-variable that binds a positive port to a negative port and
    /// is completely internal to the net.
    ///
    /// Note that a Net is linear and a bound variable will always have exactly
    /// ends with opposite polarities.
    fn bvar(&mut self, var: Self::BVar) -> Self::VarRef;
}


/// ## EquationDsl
///
pub trait EquationDsl {
    type EquationRef: Copy;
    type CellRef : Copy;
    type VarRef : Copy;

    /// A redex equation represents an interaction between the primary port of
    /// a data cell (i.e., a constructuct with a positive polarity) and the primary
    /// port of a function cell (i.e., a destructor with a negative polarity).
    fn redex(&mut self, data: Self::CellRef, fun: Self::CellRef) -> Self::EquationRef;

    /// A bind equation links a variable (free or bound) to a cell. The polarity of
    /// the variable is the opposite of the cell.
    fn bind(&mut self, var: Self::VarRef, cell: Self::CellRef) -> Self::EquationRef;

    /// A connect equation links two variables with opposite polarities (
    /// we just dont know statically which one is positive and which one is
    /// negative).
    fn connect(&mut self, left: Self::VarRef, right: Self::VarRef) -> Self::EquationRef;
}


/// ## RuleDsl

/// ### Common trait to unify type representations across Dsls
///
/// For instance, the RuleDsl needs to handle types created by the TermDsl so
/// they both define an associated type CellRef, for instance, that implements the
/// TermR trait.

/// Creates rules
pub trait RuleDsl {
    type RuleRef : Copy;
    type RuleBodyDsl : EquationDsl + TermDsl;
    type CtrSymbolRef : Copy;
    type FunSymbolRef : Copy;

    fn rule<F>(&mut self, ctr: Self::CtrSymbolRef, fun: Self::FunSymbolRef, body: F) -> Self::RuleRef
        where F: FnOnce(&mut Self::RuleBodyDsl);
}
