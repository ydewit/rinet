use crate::inet::{
    cell::CellPtr,
    equation::{EquationBuilder, EquationPtr},
    rule::RuleSet,
    symbol::{SymbolBook, SymbolName},
    term::TermPtr,
    Polarity,
};

use super::nat::{S, Z};

pub const DUP: SymbolName = SymbolName("dup");

impl SymbolBook {
    pub fn declare_combinator_symbols(&mut self) {
        self.fun2(&DUP, Polarity::Pos, Polarity::Pos);
    }
}

impl<'a> EquationBuilder<'a> {
    // Cell
    pub fn duplicator(&mut self, dup1: TermPtr, dup2: TermPtr) -> CellPtr {
        self.cell2(&DUP, dup1, dup2)
    }

    // Redex
    pub fn duplicate(&mut self, cell: TermPtr, dup1: TermPtr, dup2: TermPtr) {
        let duplicator = self.duplicator(dup1.into(), dup2.into());
        self.redex(cell.into(), duplicator.into())
    }
}

impl<'a> RuleSet<'a> {
    pub fn define_combinator_rules(&mut self) {
        // Z ⋈ dup
        self.rule(&Z, &DUP, |b| {
            let r0 = b.fun_port_0();
            let z0 = b.cell0(&Z);
            b.bind(r0.into(), z0.into());

            let r1 = b.fun_port_1();
            let z1 = b.cell0(&Z);
            b.bind(r1.into(), z1.into());
        });

        // S ⋈ dup
        self.rule(&S, &DUP, |b| {
            let (x0_input, x0_output) = b.var();
            let (x1_input, x1_output) = b.var();

            let l0 = b.ctr_port_0();
            let dup = b.cell2(&DUP, x0_input.into(), x1_input.into());
            b.bind(l0.into(), dup.into());

            let r0 = b.fun_port_0();
            let s0 = b.cell1(&S, x0_output.into());
            b.bind(r0.into(), s0.into());

            let r1 = b.fun_port_1();
            let s1 = b.cell1(&S, x1_output.into());
            b.bind(r1.into(), s1.into());
        });
    }
}
