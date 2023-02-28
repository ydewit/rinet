use core::panic;
use std::{
    fmt::{Binary, Debug, Display, Formatter, Pointer},
    sync::atomic::{AtomicU32, Ordering},
};

use tokio::sync::{
    oneshot::{channel, Receiver, Sender},
    OnceCell,
};

use super::{
    arena::{ArenaIter, ToPtr},
    cell::{CellPtr, Cells, PortPtr},
    BitSet32,
};

#[derive(Debug, PartialEq)]
pub enum VarKind {
    Bound = 0,
    Free = 1,
}

impl From<u32> for VarKind {
    fn from(value: u32) -> Self {
        match value {
            0 => VarKind::Bound,
            1 => VarKind::Free,
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct BoundStore(AtomicU32);
impl Default for BoundStore {
    fn default() -> Self {
        Self(AtomicU32::default())
    }
}
impl VarStore for BoundStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, var_ptr: VarPtr) -> std::fmt::Result {
        write!(f, "x.{}", var_ptr.get_index())
    }
}

#[derive(Debug)]
pub struct FreeStore(AtomicU32);
impl Default for FreeStore {
    fn default() -> Self {
        Self(AtomicU32::default())
    }
}
impl VarStore for FreeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, var_ptr: VarPtr) -> std::fmt::Result {
        write!(f, "_.{}", var_ptr.get_index())
    }
}

impl Display for FreeStore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
// TODO can we implement a lighter solution?
#[derive(Debug)]
pub struct FVar<S: VarStore = FreeStore>(S);

impl<S: VarStore> FVar<S> {
    pub fn new(store: S) -> Self {
        Self(store)
    }

    pub fn to_ptr(&self, index: usize) -> FVarPtr {
        FVarPtr::new(index)
    }
}

impl<S: VarStore> FVar<S> {
    pub fn get_store(&self) -> &S {
        &self.0
    }
}

impl Default for FVar {
    fn default() -> Self {
        FVar::new(FreeStore::default())
    }
}

impl<S: VarStore> ToPtr<FVarPtr> for FVar<S> {
    fn to_ptr(&self, index: usize) -> FVarPtr {
        FVarPtr::new(index)
    }
}

#[derive(Debug)]
pub struct FVars<S: VarStore = FreeStore>(Vec<FVar<S>>);
impl<S: VarStore> FVars<S> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn add(&mut self, fvar: FVar<S>) -> FVarPtr {
        let index = self.0.len();
        self.0.push(fvar);
        FVarPtr::new(index)
    }

    pub fn iter(&self) -> ArenaIter<FVar<S>, FVarPtr> {
        ArenaIter::new(&self.0)
    }

    pub fn add_all(&mut self, fvars: FVars<S>) {
        self.0.extend(fvars.0)
    }

    pub fn get(&self, ptr: FVarPtr) -> &FVar<S> {
        &self.0[ptr.get_index()]
    }

    pub fn get_mut(&mut self, ptr: FVarPtr) -> &mut FVar<S> {
        &mut self.0[ptr.get_index()]
    }
}

impl FVars<FreeStore> {
    pub fn fvar(&mut self) -> FVarPtr {
        self.add(FVar::new(FreeStore::default()))
    }

    pub fn send(&mut self, fvar_ptr: FVarPtr, cell_ptr: CellPtr) {
        let fvar = self.get_mut(fvar_ptr);
        let other_ptr = fvar.0.0.swap(cell_ptr.get_ptr(), Ordering::SeqCst);
        if other_ptr == 0 {
            panic!("already set!")
        }
    }

    pub fn try_receive(&mut self, fvar_ptr: FVarPtr) -> Option<CellPtr> {
        let fvar = self.get_mut(fvar_ptr);
        match fvar.0.0.load(Ordering::SeqCst) {
            0 => None,
            ptr => Some(CellPtr::from(ptr))
        }
    }
}

/// A BVar (bound-var as opposed to a free-var) is a synchronizing variable
/// that is used exchange a CellPtr between exactly two Bind reductions.
///
/// Note that there is no blocking or synchronization involved here except for
/// when both Bind reductions call get_or_init() at the same time. The first
/// Bind reduction will simply set the CellPtr and its task will finish execution.
/// The second Bind reduction will consume the first CellPtr and then trigger
/// a Redex reduction with both CellPtr's.
#[derive(Debug)]
pub struct BVar<S = BoundStore>(S);
impl<S> BVar<S> {
    pub fn new(store: S) -> Self {
        Self(store)
    }

    pub fn get_store(&self) -> &S {
        &self.0
    }

    pub fn to_ptr(&self, index: usize) -> BVarPtr {
        BVarPtr::new(index)
    }
}
impl Default for BVar {
    fn default() -> Self {
        BVar::new(BoundStore::default())
    }
}

impl<S> ToPtr<BVarPtr> for BVar<S> {
    fn to_ptr(&self, index: usize) -> BVarPtr {
        BVarPtr::new(index)
    }
}

#[derive(Debug)]
pub struct BVars<S = BoundStore>(Vec<BVar<S>>);
impl<S> BVars<S> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn add(&mut self, bvar: BVar<S>) -> BVarPtr {
        let index = self.0.len();
        self.0.push(bvar);
        BVarPtr::new(index)
    }

    pub fn get(&self, ptr: BVarPtr) -> &BVar<S> {
        &self.0[ptr.get_index()]
    }

    pub fn get_mut(&mut self, ptr: BVarPtr) -> &mut BVar<S> {
        &mut self.0[ptr.get_index()]
    }

    pub fn iter(&self) -> ArenaIter<BVar<S>, BVarPtr> {
        ArenaIter::new(&self.0)
    }

    pub fn add_all(&mut self, bvars: BVars<S>) {
        self.0.extend(bvars.0)
    }
}

impl BVars<BoundStore> {
    pub fn bvar(&mut self) -> BVarPtr {
        self.add(BVar::new(BoundStore::default()))
    }

    pub fn try_set(&mut self, bvar_ptr: BVarPtr, cell_ptr: CellPtr) -> Option<CellPtr> {
        let bvar = self.get_mut(bvar_ptr);
        let other_ptr = bvar.0.0.swap(cell_ptr.get_ptr(), Ordering::SeqCst);
        match other_ptr == cell_ptr.get_ptr() {
            true => None,
            false => Some(CellPtr::from(other_ptr)),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct VarPtr(u32);
impl VarPtr {
    const INDEX: BitSet32<23> = BitSet32 {
        mask: 0b00000000_01111111_11111111_11111111,
        offset: 0,
    };
    const KIND: BitSet32<1> = BitSet32 {
        mask: 0b00000000_1,
        offset: 23,
    };
    const _UNUSED: BitSet32<8> = BitSet32 {
        mask: 0b11111111,
        offset: 24,
    };

    const PTR: BitSet32<24> = BitSet32 {
        mask: 0b00000000_11111111_11111111_11111111,
        offset: 0,
    };

    fn new(index: usize, kind: VarKind) -> Self {
        assert!(index < (u32::MAX - 1) as usize);
        let mut var = Self(0);
        var.set_kind(kind); // 1-bit
        var.set_index(index); // 23-bits
        var
    }

    #[inline]
    pub fn get_ptr(&self) -> u32 {
        Self::PTR.get(self.0) // all 24-bits
    }

    #[inline]
    pub fn get_kind(&self) -> VarKind {
        VarKind::from(Self::KIND.get(self.0))
    }

    #[inline]
    fn set_kind(&mut self, kind: VarKind) {
        self.0 = Self::KIND.set(self.0, kind as u32)
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
        b.field("kind", &self.get_kind());
        b.field("index", &self.get_index());
        b.finish()
    }
}

impl Binary for VarPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Binary::fmt(&self.0, f)
    }
}

impl Into<BVarPtr> for VarPtr {
    fn into(self) -> BVarPtr {
        match self.get_kind() {
            VarKind::Free => panic!(),
            VarKind::Bound => BVarPtr(self),
        }
    }
}

impl Into<FVarPtr> for VarPtr {
    fn into(self) -> FVarPtr {
        match self.get_kind() {
            VarKind::Free => FVarPtr(self),
            VarKind::Bound => panic!(),
        }
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

#[derive(Clone, Copy, PartialEq)]
pub struct BVarPtr(VarPtr);
impl BVarPtr {
    pub fn new(index: usize) -> Self {
        Self(VarPtr::new(index, VarKind::Bound))
    }

    pub fn get_index(&self) -> usize {
        self.0.get_index()
    }

    // pub fn get_ptr(&self) -> u32 {
    //     self.0.get_ptr()
    // }
}

impl Debug for BVarPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("BVarPtr({:032b})", self.0);
        let mut b = f.debug_struct(&name);
        b.field("index", &self.get_index());
        b.finish()
    }
}

impl Default for BVarPtr {
    fn default() -> Self {
        Self(VarPtr::new(0, VarKind::Bound))
    }
}

impl From<u32> for BVarPtr {
    fn from(raw: u32) -> Self {
        Self(VarPtr::from(raw))
    }
}

impl Into<VarPtr> for BVarPtr {
    fn into(self) -> VarPtr {
        VarPtr::new(self.get_index(), VarKind::Bound)
    }
}

impl Into<PortPtr> for BVarPtr {
    fn into(self) -> PortPtr {
        PortPtr::new_var(self.0)
    }
}

impl From<PortPtr> for BVarPtr {
    fn from(value: PortPtr) -> Self {
        match value.get_kind() {
            super::cell::PortKind::Var => {
                let var = VarPtr::from(value);
                assert!(var.get_kind() == VarKind::Bound);
                BVarPtr(var)
            }
            _ => panic!(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct FVarPtr(VarPtr);
impl FVarPtr {
    pub fn new(index: usize) -> Self {
        Self(VarPtr::new(index, VarKind::Free))
    }

    pub fn get_index(&self) -> usize {
        self.0.get_index()
    }

    pub fn get_ptr(&self) -> u32 {
        self.0.get_ptr()
    }
}

impl Debug for FVarPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("FVarPtr({:032b})", self.0);
        let mut b = f.debug_struct(&name);
        b.field("index", &self.get_index());
        b.finish()
    }
}

impl From<u32> for FVarPtr {
    fn from(raw: u32) -> Self {
        Self(VarPtr::from(raw))
    }
}

impl Into<VarPtr> for FVarPtr {
    fn into(self) -> VarPtr {
        self.0
    }
}

impl Into<PortPtr> for FVarPtr {
    fn into(self) -> PortPtr {
        PortPtr::new_var(self.0)
    }
}

impl From<PortPtr> for FVarPtr {
    fn from(value: PortPtr) -> Self {
        match value.get_kind() {
            super::cell::PortKind::Var => {
                let var = VarPtr::from(value);
                assert!(var.get_kind() == VarKind::Free);
                FVarPtr(var)
            }
            _ => panic!(),
        }
    }
}

impl Binary for FVarPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Binary::fmt(&self.0 .0, f)
    }
}

pub trait VarStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, var_ptr: VarPtr) -> std::fmt::Result;
}

pub struct VarItem<'a, F: VarStore = FreeStore, B: VarStore = FreeStore> {
    pub var_ptr: VarPtr,
    pub bvars: &'a BVars<B>,
    pub fvars: &'a FVars<F>
}
impl<'a, F: VarStore, B: VarStore> Display for VarItem<'a, F, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.var_ptr.get_kind() {
            VarKind::Free => {
                let var = self.fvars.get(self.var_ptr.into());
                var.0.fmt(f, self.var_ptr.into())
            },
            VarKind::Bound => {
                let var = self.bvars.get(self.var_ptr.into());
                var.0.fmt(f, self.var_ptr.into())
            }
        }
    }
}

pub struct VarsItem<'a, F: VarStore = FreeStore, B: VarStore = BoundStore> {
    pub kind: VarKind,
    pub bvars: &'a BVars<B>,
    pub fvars: &'a FVars<F>,
}
impl<'a,F: VarStore, B: VarStore> VarsItem<'a,F,B> {
    fn to_var_item(&self, var_ptr: VarPtr) -> VarItem<'a, F, B> {
        VarItem { var_ptr, bvars: self.bvars, fvars: self.fvars }
    }
}
impl<'a, F: VarStore, B:VarStore> Display for VarsItem<'a, F, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            VarKind::Bound => {
                self.bvars.iter().fold(Ok(()), |result, ptr| {
                    result.and_then(|_| write!(f, " {}", self.to_var_item(ptr.into())))
                })
            },
            VarKind::Free => {
                self.bvars.iter().fold(Ok(()), |result, ptr| {
                    result.and_then(|_| write!(f, " {}", self.to_var_item(ptr.into())))
                })
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::inet::Polarity;

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

    #[tokio::test]
    async fn test_fvars() {
        let mut fvars = FVars::new();
        let cell = CellPtr::new(0, Polarity::Pos);
        let fvar1 = FVar::default();
        let fvar2 = FVar::default();
        let fvar3 = FVar::default();

        // Test add(), get() and add_all()
        let fvar_ptr1 = fvars.add(fvar1);
        let fvar_ptr2 = fvars.add(fvar2);
        // assert_eq!(&fvar1, fvars.get(fvar_ptr1));
        // assert_eq!(&fvar2, fvars.get(fvar_ptr2));

        let mut fvars2 = FVars::new();
        fvars2.add(fvar3);
        fvars.add_all(fvars2);
        // assert_eq!(&fvar3, fvars.get(FVarPtr::new(2)));

        // Test send() and receive()
        let fvar_ptr3 = fvars.add(FVar::new(FreeStore::default()));
        fvars.send(fvar_ptr3, cell);
        assert_eq!(Some(cell), fvars.try_receive(fvar_ptr3));
    }

    #[test]
    fn test_fvar() {
        let fvar = FVar::new(FreeStore::default());
        let fvar_ptr = fvar.to_ptr(0);
        assert_eq!(FVarPtr::new(0), fvar_ptr);
    }

    #[test]
    fn test_bvar_ptr() {
        let bvar_ptr = BVarPtr::new(0);
        assert_eq!(0, bvar_ptr.get_index());
    }

    #[test]
    fn test_fvar_ptr() {
        let fvar_ptr = FVarPtr::new(0);
        assert_eq!(0, fvar_ptr.get_index());
    }

    // #[test]
    // fn test_bound_store() {
    //     let bound_store = BoundStore::default();
    //     let cell = CellPtr::new(0, Polarity::Pos);
    //     assert_eq!(None, bound_store.0.);
    //     bound_store.0.set(cell).unwrap();
    //     assert_eq!(Some(&cell), bound_store.0.get());
    // }

    // #[test]
    // fn test_free_store() {
    //     let free_store = FreeStore::default();
    //     let cell = CellPtr::new(0, Polarity::Pos);
    //     let (sender, receiver) = channel();
    //     assert_eq!(sender, free_store.0);
    //     assert_eq!(receiver, free_store.1);
    // }
}
