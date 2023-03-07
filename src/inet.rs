use std::{
    fmt::Display,
    ops::{BitAnd, BitOr, Shl, Shr},
};

pub mod arena;
pub mod cell;
pub mod equation;
pub mod heap;
pub mod net;
pub mod rule;
pub mod runtime;
pub mod symbol;
pub mod term;
pub mod util;
pub mod var;

/// ## Polarity
///
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Polarity {
    Pos = 0,
    Neg = 1,
}

impl Polarity {
    const MAX: u8 = 0b1;
}

impl From<u32> for Polarity {
    fn from(value: u32) -> Self {
        match value {
            0 => Polarity::Pos,
            1 => Polarity::Neg,
            _ => panic!(),
        }
    }
}

impl From<u16> for Polarity {
    fn from(value: u16) -> Self {
        match value {
            0 => Polarity::Pos,
            1 => Polarity::Neg,
            _ => panic!(),
        }
    }
}

impl From<u8> for Polarity {
    fn from(value: u8) -> Self {
        match value {
            0 => Polarity::Pos,
            1 => Polarity::Neg,
            _ => panic!(),
        }
    }
}

impl From<usize> for Polarity {
    fn from(value: usize) -> Self {
        match value {
            0 => Polarity::Pos,
            1 => Polarity::Neg,
            _ => panic!(),
        }
    }
}

impl From<u64> for Polarity {
    fn from(value: u64) -> Self {
        match value {
            0 => Polarity::Pos,
            1 => Polarity::Neg,
            _ => panic!(),
        }
    }
}

impl Display for Polarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Polarity::Pos => write!(f, "+",),
            Polarity::Neg => write!(f, "-",),
        }
    }
}

pub trait Polarized {
    fn polarity(&self) -> Polarity;
}

impl Polarity {
    fn flip(&self) -> Polarity {
        match self {
            Polarity::Pos => Polarity::Neg,
            Polarity::Neg => Polarity::Pos,
        }
    }

    fn is_opposite(&self, pol: Polarity) -> bool {
        self != &pol
    }
}

/// ## BitField
/// Experimental implementation of BitSet that is polymorphic
///
pub struct BitField<
    T: Copy + PartialOrd + Shl<Output = T> + Shr<Output = T> + BitAnd<Output = T>,
    const N: usize,
> {
    pub mask: T,
    pub offset: T,
}

impl<
        T: Copy
            + PartialOrd
            + Shl<Output = T>
            + Shr<Output = T>
            + BitAnd<Output = T>
            + BitOr<Output = T>,
        const N: usize,
    > BitField<T, N>
{
    pub fn check_value(&self, value: &T) {
        // assert!(self.mask < (1 << self.len()));
        // assert!(value >>  (1 << self.len()));
    }

    #[inline]
    pub fn len(&self) -> usize {
        N
    }

    #[inline]
    pub fn set(&self, bits: T, value: T) -> T {
        self.check_value(&value);
        bits | ((value & self.mask) << self.offset)
    }

    #[inline]
    pub fn get(&self, value: T) -> T {
        (value >> self.offset) & self.mask
    }
}

pub struct BitSet64<const N: usize> {
    mask: u64,
    offset: u8,
}

impl<const N: usize> BitSet64<N> {
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

pub struct Bits64<const N: usize>(u64);
pub struct Bits32<const N: usize>(u32);
pub struct Bits16<const N: usize>(u16);
pub struct Bits8<const N: usize>(u8);

impl<const N: usize> Bits64<N> {
    pub fn new(value: u64) -> Self
    where
        Assert<{ N < 8 }>: IsTrue,
    {
        Self(value)
    }
}

pub enum Assert<const CHECK: bool> {}

pub trait IsTrue {}

impl IsTrue for Assert<true> {}

pub struct BitSet32<const N: usize> {
    mask: u32,
    offset: u8,
}

impl<const N: usize> BitSet32<N> {
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

    #[inline]
    pub fn len(&self) -> usize {
        N
    }
}

pub struct BitSet16<const N: usize> {
    mask: u16,
    offset: u8,
}

impl<const N: usize> BitSet16<N> {
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
}

pub struct BitSet8<const N: usize> {
    mask: u8,
    offset: u8,
}

impl<const N: usize> BitSet8<N> {
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

// mod fibonacci {
//     use crate::net::term::{Equation, CellTag};

//     struct Rule {
//     }

//     impl Rule {
//         pub fn fib_z_rule(left: [char;1], right: [char;1], bvars: [char;4]) -> (Equation, Ptr<CellTag>) {

//         }
//     }

// }
