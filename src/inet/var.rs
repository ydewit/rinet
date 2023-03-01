use core::panic;
use std::fmt::{Binary, Debug, Display, Formatter};

use super::{
    arena::{ArenaIter, ToPtr},
    cell::PortPtr,
    term::TermFamily,
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

impl Into<PortPtr> for VarPtr {
    fn into(self) -> PortPtr {
        PortPtr::new_var(self)
    }
}

impl From<u32> for VarPtr {
    fn from(raw: u32) -> Self {
        Self(raw)
    }
}

impl From<PortPtr> for VarPtr {
    fn from(value: PortPtr) -> Self {
        match value.get_kind() {
            super::cell::PortKind::Var => VarPtr(value.get_ptr()),
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct Var<T: TermFamily>(pub T::Store);
impl<T: TermFamily> Var<T> {
    pub fn new(store: T::Store) -> Self {
        Self(store)
    }

    pub fn to_ptr(&self, index: usize) -> VarPtr {
        VarPtr::new(index)
    }
}

#[derive(Debug)]
pub struct Vars<T: TermFamily>(Vec<Var<T>>);
impl<T: TermFamily> Vars<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn add(&mut self, var: Var<T>) -> VarPtr {
        let index = self.0.len();
        self.0.push(var);
        VarPtr::new(index)
    }

    pub fn iter(&self) -> ArenaIter<Var<T>, VarPtr> {
        ArenaIter::new(&self.0)
    }

    pub fn add_all(&mut self, fvars: Vars<T>) {
        self.0.extend(fvars.0)
    }

    pub fn get(&self, ptr: VarPtr) -> &Var<T> {
        &self.0[ptr.get_index()]
    }
}

impl<T: TermFamily> ToPtr<VarPtr> for Var<T> {
    fn to_ptr(&self, index: usize) -> VarPtr {
        VarPtr::new(index)
    }
}

pub struct VarItem<'a, T: TermFamily> {
    pub var_ptr: VarPtr,
    pub vars: &'a Vars<T>,
}
impl<'a, T: TermFamily> Display for VarItem<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let var = self.vars.get(self.var_ptr.into());
        T::display_store(f, &var.0, self.var_ptr.get_index())
    }
}

pub struct VarsItem<'a, T: TermFamily> {
    pub vars: &'a Vars<T>,
}
impl<'a, T: TermFamily> VarsItem<'a, T> {
    fn to_var_item(&self, var_ptr: VarPtr) -> VarItem<'a, T> {
        VarItem {
            var_ptr,
            vars: self.vars,
        }
    }
}
impl<'a, T: TermFamily> Display for VarsItem<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.vars.iter().fold(Ok(()), |result, ptr| {
            result.and_then(|_| write!(f, " {}", self.to_var_item(ptr.into())))
        })
    }
}

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
