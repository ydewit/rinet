use std::fmt::{Binary, Debug, Formatter};

use super::{
    arena::{Arena, ArenaPtr, ArenaValue},
    term::TermFamily,
    BitSet32, Polarity,
};

#[derive(Debug)]
pub struct PVarPtr(u32);
impl PVarPtr {
    const POLARITY: BitSet32<1> = BitSet32 {
        mask: 0b00000000_1,
        offset: 23,
    };

    const VAR_PTR: BitSet32<23> = BitSet32 {
        mask: 0b00000000_01111111_11111111_11111111,
        offset: 0,
    };

    const PTR: BitSet32<24> = BitSet32 {
        mask: 0b00000000_11111111_11111111_11111111,
        offset: 0,
    };

    fn new(var_ptr: VarPtr, polarity: Polarity) -> PVarPtr {
        let mut pvar_ptr = Self(var_ptr.get_ptr());
        pvar_ptr.set_polarity(polarity);
        pvar_ptr
    }

    pub fn wire(var_ptr: VarPtr) -> (PVarPtr, PVarPtr) {
        let in_ptr = Self::new(var_ptr, Polarity::Neg);
        let out_ptr = Self::new(var_ptr, Polarity::Pos);
        (in_ptr, out_ptr)
    }

    pub fn get_polarity(&self) -> Polarity {
        Polarity::from(Self::POLARITY.get(self.0))
    }

    fn set_polarity(&mut self, polarity: Polarity) {
        self.0 = Self::POLARITY.set(self.0, polarity as u32)
    }

    pub fn get_fvar_ptr(&self) -> VarPtr {
        VarPtr(Self::VAR_PTR.get(self.0))
    }

    pub fn get_ptr(&self) -> u32 {
        Self::PTR.get(self.0)
    }
}

impl Into<VarPtr> for PVarPtr {
    fn into(self) -> VarPtr {
        self.get_fvar_ptr()
    }
}

impl Into<VarPtr> for &PVarPtr {
    fn into(self) -> VarPtr {
        self.get_fvar_ptr()
    }
}

impl From<u32> for PVarPtr {
    fn from(value: u32) -> Self {
        PVarPtr(value)
    }
}

/// # PVarPtrBuffer
///

pub struct PVarPtrBuffer {
    buffer: [VarPtr; Self::MAX_BUFFER_LEN],
    len: u8,
}

impl PVarPtrBuffer {
    const MAX_BUFFER_LEN: usize = 10;

    pub fn new(len: u8) -> Self {
        assert!(Self::MAX_BUFFER_LEN > len as usize);
        Self {
            len,
            buffer: [
                VarPtr(0),
                VarPtr(0),
                VarPtr(0),
                VarPtr(0),
                VarPtr(0),
                VarPtr(0),
                VarPtr(0),
                VarPtr(0),
                VarPtr(0),
                VarPtr(0),
            ],
        }
    }

    pub fn set(&mut self, index: u8, var_ptr: VarPtr) {
        assert!(index < self.len);
        self.buffer[index as usize] = var_ptr;
    }

    pub fn get_neg_var(&self, index: u8) -> PVarPtr {
        assert!(index < self.len);
        PVarPtr::new(self.buffer[index as usize], Polarity::Neg)
    }

    pub fn get_pos_var(&self, index: u8) -> PVarPtr {
        assert!(index < self.len);
        PVarPtr::new(self.buffer[index as usize], Polarity::Pos)
    }
}

/// # VarPtr

#[derive(Clone, Copy, PartialEq)]
pub struct VarPtr(u32);
impl VarPtr {
    const INDEX: BitSet32<23> = BitSet32 {
        mask: 0b00000000_01111111_11111111_11111111,
        offset: 0,
    };
    const _UNUSED: BitSet32<1> = BitSet32 {
        mask: 0b11111111_1,
        offset: 23,
    };

    const PTR: BitSet32<23> = BitSet32 {
        mask: 0b00000000_01111111_11111111_11111111,
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

    #[inline]
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

impl From<u32> for VarPtr {
    fn from(raw: u32) -> Self {
        Self(raw)
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

    pub fn to_ptr(&self, index: usize) -> VarPtr {
        VarPtr::new(index)
    }
}

impl<T: TermFamily> ArenaValue<VarPtr> for Var<T> {
    fn to_ptr(&self, index: usize) -> VarPtr {
        VarPtr::new(index)
    }
}

pub type Vars<T> = Arena<Var<T>, VarPtr>;

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
