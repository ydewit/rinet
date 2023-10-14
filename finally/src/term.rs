use crate::symbol::SymbolSym;

pub trait Port {}

pub trait CellSym: SymbolSym {
    type Cell: Port;

    fn cell0(symbol: Self::Symbol) -> Self::Cell;
    fn cell1(symbol: Self::Symbol, port: impl Port) -> Self::Cell;
    fn cell2(symbol: Self::Symbol, left: impl Port, right: impl Port) -> Self::Cell;
}

pub trait VarSym {
    type Var: Port;

    fn var<F>(scope: F) -> Self::Var
    where
        F: Fn();
}
