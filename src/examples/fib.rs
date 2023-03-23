use crate::inet::{
    cell::CellPtr,
    equation::EquationBuilder,
    net::Net,
    rule::{RuleBuilder, RuleSet},
    symbol::{SymbolBook, SymbolName},
    term::TermPtr,
    Polarity,
};

use super::{
    combinators::DUP,
    nat::{S, Z},
};

pub const FIB: SymbolName = SymbolName("fib");
const FIB_0: SymbolName = SymbolName("fib₀");

impl SymbolBook {
    pub fn declare_fib_symbols(&mut self) {
        self.fun1(&FIB, Polarity::Pos);
        self.fun1(&FIB_0, Polarity::Pos);
    }
}

impl<'a> EquationBuilder<'a> {
    pub fn fib(&mut self, num: TermPtr) -> CellPtr {
        self.cell1(&FIB, num.into())
    }

    pub fn fib0(&mut self, num: TermPtr) -> CellPtr {
        self.cell1(&FIB_0, num.into())
    }

    pub fn fibonacci(&mut self, num: TermPtr, result: TermPtr) {
        let fib = self.cell1(&FIB, result);
        self.redex(num.into(), fib.into())
    }
}

impl<'a, 'b> RuleBuilder<'a, 'b> {
    pub fn fib(&mut self, num: TermPtr) -> CellPtr {
        self.cell1(&FIB, num.into())
    }

    pub fn fib0(&mut self, num: TermPtr) -> CellPtr {
        self.cell1(&FIB_0, num.into())
    }
}

impl<'a> RuleSet<'a> {
    pub fn fib_rules(&mut self) {
        // Z ⋈ (fib r₀)  ⟶ r₀ ← Z
        self.rule(&Z, &FIB, |b| {
            let r0 = b.fun_port_0();
            // let add1 = b.fun_port_1(); // ERROR
            let zero = b.zero();
            b.bind(r0.into(), zero.into());
        });

        // (S l₀) ⋈ (fib r₀)  ⟶  l₀ = (fib₀ r₀)
        self.rule(&S, &FIB, |b| {
            let r0 = b.fun_port_0();
            let fib0 = b.fib0(r0.into());

            let l0 = b.ctr_port_0();
            b.bind(l0.into(), fib0.into());
        });

        // Z ⋈ (fib₀ r₀)  ⟶ r₀ ← (S Z)
        self.rule(&Z, &FIB_0, |b| {
            let r0 = b.fun_port_0();
            let one = b.one();
            b.bind(r0.into(), one.into());
        });

        // (S l₀) ⋈ (fib₀ r₀)  ⟶  x₀ ← (fib₀ x₂); x₁ ← (fib x₃); l₀ ← (dup x₀ x₁); x₂ ← (add x₃ r₀)
        self.rule(&S, &FIB_0, |b| {
            let (x0_in, x0_out) = b.var();
            let (x1_in, x1_out) = b.var();
            let (x2_in, x2_out) = b.var();
            let (x3_in, x3_out) = b.var();

            // ⟨ .cell ⟨"dup", #[.var (.bvar 0), .var (.bvar 1)]⟩, .var (.fvar (.inr 0)) ⟩
            let l0 = b.ctr_port_0();
            let dup = b.cell2(&DUP, x0_in.into(), x1_in.into());
            b.bind(l0.into(), dup.into());

            // ⟨ .cell ⟨"add", #[.var (.bvar 3), .var (.fvar (.inl 0))]⟩, .var (.bvar 2)⟩
            let r0 = b.fun_port_0();
            let adder = b.adder(r0.into(), x3_out.into());
            b.bind(x2_out.into(), adder.into());

            // ⟨ .cell ⟨"fib₀", #[.var (.bvar 2)]⟩, .var (.bvar 0) ⟩
            let fib0 = b.fib0(x2_in.into());
            b.bind(x0_out.into(), fib0.into());

            // ⟨ .cell ⟨"fib", #[.var (.bvar 3)]⟩, .var (.bvar 1) ⟩
            let fib = b.fib(x3_in.into());
            b.bind(x1_out.into(), fib.into());
        });
    }
}

impl<'a> Net<'a> {
    pub fn fib(&mut self, n: usize) {
        self.equations(|b| {
            let result = b.output_fvar();
            let num = b.n(n);
            b.fibonacci(num.into(), result.into());
        })
    }
}
