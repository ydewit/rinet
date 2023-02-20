use std::fmt::Display;

pub mod arena;
pub mod cell;
pub mod equation;
pub mod net;
pub mod symbol;
pub mod var;
pub mod rule;

/// ## Polarity
///
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Polarity {
    Pos = 0,
    Neg = 1
}

impl Polarity {
    const MAX : u8 = 0b1;
}

impl From<u32> for Polarity {
    fn from(value: u32) -> Self {
        match value {
            0 => Polarity::Pos,
            1 => Polarity::Neg,
            _ => panic!()
        }
    }
}

impl From<u16> for Polarity {
    fn from(value: u16) -> Self {
        match value {
            0 => Polarity::Pos,
            1 => Polarity::Neg,
            _ => panic!()
        }
    }
}

impl From<u8> for Polarity {
    fn from(value: u8) -> Self {
        match value {
            0 => Polarity::Pos,
            1 => Polarity::Neg,
            _ => panic!()
        }
    }
}

impl From<usize> for Polarity {
    fn from(value: usize) -> Self {
        match value {
            0 => Polarity::Pos,
            1 => Polarity::Neg,
            _ => panic!()
        }
    }
}

impl From<u64> for Polarity {
    fn from(value: u64) -> Self {
        match value {
            0 => Polarity::Pos,
            1 => Polarity::Neg,
            _ => panic!()
        }
    }
}

impl Display for Polarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Polarity::Pos => write!(f, "+", ),
            Polarity::Neg => write!(f, "-", ),
        }

}
}

pub trait Polarized {
    fn polarity(&self) -> Polarity ;
}

impl Polarity {
    fn flip (&self) -> Polarity {
        match self {
            Polarity::Pos => Polarity::Neg,
            Polarity::Neg => Polarity::Pos
        }
    }
}



pub struct BitSet64 {
    mask: u64,
    offset: u8
}

impl BitSet64 {
    #[inline]
    pub fn new(mask: u64, offset: u8) -> Self {
        assert!(offset <= 64);
        Self { mask, offset }
    }

    #[inline]
    pub fn set(&self, bits: u64, value: u64) -> u64 {
        assert!(value < (self.mask << self.offset));
        bits | ((value & self.mask) << self.offset)
    }

    #[inline]
    pub fn get(&self, bits: u64) -> u64 {
        (bits >> self.offset) & self.mask
    }
}

pub struct BitSet32 {
    mask: u32,
    offset: u8
}

impl BitSet32 {
    #[inline]
    pub fn new(mask: u32, offset: u8) -> Self {
        assert!(offset <= 32);
        Self { mask, offset }
    }

    #[inline]
    pub fn set(&self, bits: u32, value: u32) -> u32 {
        assert!(value < (self.mask << self.offset));
        bits | ((value & self.mask) << self.offset)
    }

    #[inline]
    pub fn get(&self, bits: u32) -> u32 {
        (bits >> self.offset) & self.mask
    }
}

pub struct BitSet16 {
    mask: u16,
    offset: u8
}

impl BitSet16 {
    #[inline]
    pub fn new(mask: u16, offset: u8) -> Self {
        assert!(offset <= 16);
        Self { mask, offset }
    }

    #[inline]
    pub fn set(&self, bits: u16, value: u16) -> u16 {
        assert!(value < (self.mask << self.offset));
        bits | ((value & self.mask) << self.offset)
    }

    #[inline]
    pub fn get(&self, bits: u16) -> u16 {
        (bits >> self.offset) & self.mask
    }

    pub fn assert_no_overlap(&self, other: &BitSet16) {
        todo!()
    }
}

pub struct BitSet8 {
    mask: u8,
    offset: u8
}


impl BitSet8 {
    #[inline]
    pub fn new(mask: u8, offset: u8) -> Self {
        assert!(offset <= 8);
        Self { mask, offset }
    }

    #[inline]
    pub fn set(&self, bits: u8, value: u8) -> u8 {
        assert!(value < (self.mask << self.offset));
        bits | ((value & self.mask) << self.offset)
    }

    #[inline]
    pub fn get(&self, bits: u8) -> u8 {
        (bits >> self.offset) & self.mask
    }
}

// ---------------------



use core::panic;

use crate::inet::{symbol::SymbolBook, rule::RuleBook, net::{Net, CellItem}};

pub fn inet_main() {
    // symbols
    let mut symbols = SymbolBook::new();
    let z_sym = symbols.add_symbol0("Z", Polarity::Pos);
    let s_sym = symbols.add_symbol1("S", Polarity::Pos, Polarity::Neg);
    let add_sym = symbols.add_symbol2("add", Polarity::Neg, Polarity::Pos, Polarity::Neg);
    println!("{}", symbols);

    // net
    let mut net = Net::new(&symbols);
    let z = net.cell0(z_sym);
    let s = net.cell1(s_sym, z.into());
    let r_fvar = net.fvar();
    let add = net.cell2(add_sym, r_fvar.into(), s.into());

    let eqn_ptr = net.redex(s.into(), add.into());
    let eqn = net.get_equation(eqn_ptr);
    let right = eqn.get_redex_right();
    println!("{:?}", right);
    println!("{}", net.display_cell(right));

    println!("{}", net);
    println!();
    println!("{:?}", net);


    // rules

    // Z >< add
    let mut rules = RuleBook::new();
    let z_add = rules.new_rule(z_sym, add_sym, |builder|{
        let add0 = builder.fun_port_0();
        let add1 = builder.fun_port_1();
        builder.connect(add0.into(), add1.into());
    });
    // S >< add
    let s_add = rules.new_rule(s_sym, add_sym, |builder|{
        let x = builder.var();
        let add__port_0 = builder.fun_port_0();
        let S_x = builder.cell1(s_sym, x.into());
        builder.bind(add__port_0.into(), S_x);

        let z = builder.cell0(z_sym);
        let add = builder.cell2(add_sym, x.into(), z.into());

        let s_port_0 = builder.ctr_port_0();
        builder.bind(s_port_0.into(), add);
    });

    println!();
    println!("{:?}", rules);

    // // let net = Arc::new(net);
    // match eqn.get_kind() {
    //     EquationKind::Redex => {
    //         // spawn(async {
    //             let _rule = todo!();
    //             let left_ptr = net.get_cell(eqn.get_redex_left());
    //             let right_ptr = net.get_cell(eqn.get_redex_right());

    //         // });
    //     },
    //     EquationKind::Bind => {
    //         let var = eqn.get_bind_var();
    //         let cell = eqn.get_bind_cell();
    //         match cell.get_polarity() {
    //             Polarity::Pos => {
    //                 if var.is_free() {
    //                     net.send(var.into(), cell);
    //                 }
    //                 else {
    //                     let cell = net.receive(var.into());

    //                 }
    //             },
    //             Polarity::Neg => todo!(),
    //         }
    //     },
    //     EquationKind::Connect => {
    //         panic!()
    //     },
    // }

}