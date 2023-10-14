
use crate::{symbol::SymbolSym, net::EquationSym};


// enum FunPort {
//     Zero,
//     One
// }

// enum CtrPort {
//     Zero,
//     One
// }

pub trait RuleSym: EquationSym + SymbolSym {
    type Rule;
    type RuleBuilder;

    fn rule<F>(left: Self::Symbol, right: Self::Symbol) -> Self::RuleBuilder
    where
        F: Fn(Self::Equation);
}
