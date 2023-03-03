use core::panic;
use std::fmt::{Binary, Debug, Formatter};

use super::{
    arena::{Arena, ArenaPtr, ArenaValue},
    term::{TermFamily, TermKind, TermPtr},
    BitSet32,
};

#[derive(Clone, Copy, PartialEq)]
pub struct VarPtr(u32);
impl VarPtr {
    const INDEX: BitSet32<23> = BitSet32 {
        mask: 0b00000000_11111111_11111111_11111111,
        offset: 0,
    };
    const _UNUSED: BitSet32<8> = BitSet32 {
        mask: 0b11111111,
        offset: 24,
    };

    const PTR: BitSet32<24> = BitSet32 {
        mask: 0b00000000_11111111_11111111_11111111,
        offset: 0,
    };

    pub fn new(index: usize) -> Self {
        assert!(index < (u32::MAX - 1) as usize);
        let mut var = Self(0);
        var.set_index(index); // 23-bits
        var
    }

    #[inline]
    pub fn get_ptr(&self) -> u32 {
        Self::PTR.get(self.0) // all 24-bits
    }

    #[inline]
    pub fn get_index(&self) -> usize {
        Self::INDEX.get(self.0) as usize
    }

    fn set_index(&mut self, index: usize) {
        self.0 = Self::INDEX.set(self.0, index as u32)
    }
}

impl ArenaPtr for VarPtr {
    fn get_index(&self) -> usize {
        self.get_index()
    }
}

impl Debug for VarPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("VarPtr({:0b})", self.0);
        let mut b = f.debug_struct(&name);
        b.field("index", &self.get_index());
        b.finish()
    }
}

impl Binary for VarPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Binary::fmt(&self.0, f)
    }
}

impl Into<TermPtr> for VarPtr {
    fn into(self) -> TermPtr {
        TermPtr::new_var(self)
    }
}

impl From<u32> for VarPtr {
    fn from(raw: u32) -> Self {
        Self(raw)
    }
}

impl From<TermPtr> for VarPtr {
    fn from(value: TermPtr) -> Self {
        match value.get_kind() {
            TermKind::Var => VarPtr(value.get_ptr()),
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub enum Var<T: TermFamily> {
    Bound(T::BoundStore),
    Free(T::FreeStore),
}

impl<T: TermFamily> Var<T> {
    pub fn bvar(store: T::BoundStore) -> Self {
        Self::Bound(store)
    }

    pub fn fvar(store: T::FreeStore) -> Self {
        Self::Free(store)
    }

    pub fn is_bound(&self) -> bool {
        match self {
            Var::Bound(_) => true,
            _ => false,
        }
    }

    pub fn is_free(&self) -> bool {
        match self {
            Var::Free(_) => true,
            _ => false,
        }
    }

    // pub fn get_store(&self) -> &T::Store {
    //     match self {
    //         Var::Bound(store) => store,
    //         Var::Free(store) => store,
    //     }
    // }

    pub fn to_ptr(&self, index: usize) -> VarPtr {
        VarPtr::new(index)
    }
}

impl<T: TermFamily> ArenaValue<VarPtr> for Var<T> {
    fn to_ptr(&self, index: usize) -> VarPtr {
        VarPtr::new(index)
    }
}

pub type Vars<T: TermFamily> = Arena<Var<T>, VarPtr>;

#[cfg(test)]
mod tests {
    use super::*;

    // #[tokio::test]
    // async fn test_bvar() {
    //     let mut bvar = BVar::default();
    //     let cell1 = CellPtr::new(0, Polarity::Pos);
    //     let cell2 = CellPtr::new(0, Polarity::Neg);

    //     // Test get_or_init() when the bvar is uninitialized
    //     assert_eq!(None, bvar.try_set(cell1));
    //     assert_eq!(Some(cell1), bvar.try_set(cell2));

    //     // Test get_or_init() when the bvar is already initialized
    //     // assert_eq!(Some(cell1), bvar.get_store().try_get());
    //     assert_eq!(Some(cell2), bvar.try_set(cell2));
    // }
}
