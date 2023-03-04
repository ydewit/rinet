use crate::inet::{
    cell::CellPtr,
    equation::EquationBuilder,
    symbol::{SymbolBook, SymbolName},
    Polarity, rule::RuleBuilder,
};

// Nats

pub const Z: SymbolName = SymbolName("Z");
pub const S: SymbolName = SymbolName("S");

impl SymbolBook {
    pub fn declare_nat_symbols(&mut self) {
        // Z
        self.ctr0(&Z);

        // S
        self.ctr1(&S, Polarity::Neg);
    }
}

impl<'a, 'b> RuleBuilder<'a, 'b> {
    pub fn zero(&mut self) -> CellPtr {
        self.cell0(&Z)
    }

    pub fn one(&mut self) -> CellPtr {
        let zero = self.zero();
        self.cell1(&S, zero.into())
    }

    pub fn succ(&mut self, succ: CellPtr) -> CellPtr {
        self.cell1(&S, succ.into()) // TODO how to enforce typing?
    }

    pub fn two(&mut self) -> CellPtr {
        let one = self.one();
        self.succ(one.into())
    }
}


impl<'a> EquationBuilder<'a> {
    pub fn zero(&mut self) -> CellPtr {
        self.cell0(&Z)
    }

    pub fn one(&mut self) -> CellPtr {
        let zero = self.zero();
        self.cell1(&S, zero.into())
    }

    pub fn succ(&mut self, succ: CellPtr) -> CellPtr {
        self.cell1(&S, succ.into()) // TODO how to enforce typing?
    }

    pub fn two(&mut self) -> CellPtr {
        let one = self.one();
        self.succ(one.into())
    }
}
