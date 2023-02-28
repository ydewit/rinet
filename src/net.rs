use std::fmt::Display;

use self::arena::Ptr;


// pub mod arena;
// pub mod heap;
// pub mod channel;
// pub mod finally;
pub mod dsl;
pub mod symbol;
pub mod term;
pub mod rule;
pub mod runtime;
pub mod arena;
pub mod main;

impl<T: Copy + Polarized> Polarized for Ptr<T> {
    fn polarity(&self) -> Polarity {
        self.tag.polarity()
    }
}

/// ## Polarity
///
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Polarity {
    Pos = 0,
    Neg = 1
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
