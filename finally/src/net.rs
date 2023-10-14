use crate::term::{VarSym, CellSym};

pub trait EquationSym: VarSym + CellSym {
    type Equation;

    fn redex(fun: Self::Cell, ctr: Self::Cell) -> Self::Equation;
    fn bind(var: Self::Var, cell: Self::Cell) -> Self::Equation;
    fn connect(left: Self::Var, right: Self::Var) -> Self::Equation;
}
pub trait NetSym: EquationSym {
    type Net;

    fn inp<F>(scope: F) -> Self::Net
    where
        F: Fn(Self::Var);

    fn out<F>(scope: F) -> Self::Net
    where
        F: Fn(Self::Var);
}
