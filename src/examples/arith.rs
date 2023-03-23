use crate::inet::{
    cell::CellPtr,
    equation::{EquationBuilder, EquationPtr},
    rule::{RuleBuilder, RuleSet},
    symbol::{SymbolBook, SymbolName},
    term::TermPtr,
    Polarity,
};

use super::nat::{S, Z};

pub const ADD: SymbolName = SymbolName("add");
pub const SUB: SymbolName = SymbolName("sub");
const SUB_0: SymbolName = SymbolName("sub₀");

impl SymbolBook {
    pub fn declare_arith_symbols(&mut self) {
        // add
        self.fun2(&ADD, Polarity::Pos, Polarity::Neg);

        // sub
        self.fun2(&SUB_0, Polarity::Pos, Polarity::Neg);
        self.fun2(&SUB, Polarity::Pos, Polarity::Neg);
    }
}

impl<'a> EquationBuilder<'a> {
    // adder function
    pub fn adder(
        &mut self,
        result: TermPtr,   // port 0
        operand2: TermPtr, // port 1
    ) -> CellPtr {
        self.cell2(&ADD, result.into(), operand2.into())
    }

    // subtractor function
    pub fn subtractor(
        &mut self,
        result: TermPtr,   // port 0
        operand2: TermPtr, // port 1
    ) -> CellPtr {
        self.cell2(&SUB, result.into(), operand2.into())
    }

    // add redex
    pub fn add(&mut self, operand1: CellPtr, adder: CellPtr) {
        self.redex(operand1, adder)
    }

    // subtract redex
    pub fn subtract(
        &mut self,
        operand1: CellPtr,   // constructor
        subtractor: CellPtr, // function
    ) {
        self.redex(operand1, subtractor)
    }
}

impl<'a, 'b> RuleBuilder<'a, 'b> {
    pub fn adder(
        &mut self,
        result: TermPtr,   // port 0
        operand2: TermPtr, // port 1
    ) -> CellPtr {
        self.cell2(&ADD, result.into(), operand2.into())
    }

    // subtractor function
    pub fn subtractor(
        &mut self,
        result: TermPtr,   // port 0
        operand2: TermPtr, // port 1
    ) -> CellPtr {
        self.cell2(&SUB, result.into(), operand2.into())
    }
}

impl<'a> RuleSet<'a> {
    pub fn arith_rules(&mut self) {
        self.arith_add_rules();
        self.arith_sub_rules();
    }

    pub fn arith_add_rules(&mut self) {
        // (add x₁ x₂)=Z  ⟶  x₁ = x₂
        self.rule(&Z, &ADD, |b| {
            let add0 = b.fun_port_0();
            let add1 = b.fun_port_1();
            b.connect(add0.into(), add1.into());
        });

        // (add x₁ x₂)=(S n)  ⟶  (add X x₂) ⋈ n
        self.rule(&S, &ADD, |b| {
            let (neg_pvar, pos_pvar) = b.var();
            let fun_0 = b.fun_port_0();
            let S_x = b.cell1(&S, pos_pvar.into());
            b.bind(fun_0.into(), S_x);

            let fun_1 = b.fun_port_1();
            let add = b.cell2(&ADD, neg_pvar.into(), fun_1.into());

            let s_port_0 = b.ctr_port_0();
            b.bind(s_port_0.into(), add);
        });
    }

    pub fn arith_sub_rules(&mut self) {
        // (Sub l0 l1) = Z       ⟶  l0 = l1
        self.rule(&Z, &SUB, |b| {
            let l0 = b.fun_port_0();
            let l1 = b.fun_port_1();
            b.connect(l0.into(), l1.into());
        });

        // (Sub l0 l1) = (S r0)  ⟶  (Sub₀ l0 r0) = l1
        self.rule(&S, &SUB, |b| {
            let l0 = b.fun_port_0();
            let l1 = b.fun_port_1();
            let r0 = b.ctr_port_0();

            let sub0 = b.cell2(&SUB_0, l0.into(), r0.into());
            b.bind(l1.into(), sub0.into());
        });

        // (Sub₀ l0 l1) = Z       ⟶  l0 = (S l1)
        self.rule(&Z, &SUB_0, |b| {
            let l0 = b.fun_port_0();
            let l1 = b.fun_port_1();

            let s = b.cell1(&S, l1.into());
            b.bind(l0.into(), s.into());
        });

        // (Sub₀ l0 l1) = (S r0)  ⟶  (Sub l0 r0) = l1
        self.rule(&S, &SUB_0, |b| {
            let l0 = b.fun_port_0();
            let l1 = b.fun_port_1();
            let r0 = b.ctr_port_0();

            let sub = b.cell2(&SUB, l0.into(), r0.into());
            b.bind(l1.into(), sub.into());
        });
    }
}
