use core::panic;
use std::fmt::{Formatter, Debug};

use tokio::sync::{OnceCell, oneshot::{Receiver, Sender, channel}};

use super::{cell::{CellPtr, PortPtr}, arena::{ToPtr, ArenaIter}, BitSet32};

#[derive(Debug)]
pub struct BoundStore(OnceCell<CellPtr>);
impl Default for BoundStore {
    fn default() -> Self {
        Self(OnceCell::default())
    }
}

#[derive(Debug)]
pub struct FreeStore(Sender<CellPtr>, Receiver<CellPtr>);
impl Default for FreeStore {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self(sender, receiver)
    }
}

// TODO can we implement a lighter solution?
#[derive(Debug)]
pub struct FVar<S = FreeStore>(S);

impl<S> FVar<S> {
    pub fn new(store: S) -> Self {
        Self(store)
    }

    // pub fn send(&mut self, cell: CellPtr) -> bool {
    //     match self.0.send(cell) {
    //         Ok(_) => true,
    //         Err(_) => false,
    //     }
    // }

    // pub fn receive(self) -> CellPtr {
    //     match self.1.blocking_recv() {
    //         Ok(cell) => cell,
    //         Err(_) => panic!(),
    //     }
    // }

    pub fn to_ptr(&self, index: usize) -> FVarPtr {
        FVarPtr::new(index)
    }
}

impl Default for FVar {
    fn default() -> Self {
        FVar::new(FreeStore::default())
    }
}

impl<S> ToPtr<FVarPtr> for FVar<S> {
    fn to_ptr(&self, index: usize) -> FVarPtr {
        FVarPtr::new(index)
    }
}

#[derive(Debug)]
pub struct FVars<S = FreeStore>(Vec<FVar<S>>);
impl<S> FVars<S> {
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

    pub fn iter(&self) -> ArenaIter<FVar<S>,FVarPtr> {
        ArenaIter::new(&self.0)
    }

    pub fn add_all(&mut self, fvars: FVars<S>) {
        self.0.extend(fvars.0)
    }

    pub fn get(&self, ptr: FVarPtr) -> &FVar<S> {
        &self.0[ptr.get_index()]
    }

    pub fn send(&self, fvar: FVarPtr, cell: CellPtr) -> bool {
        let var = &self.0[fvar.get_index()];
        // var.send(cell) // TODO fix me
        false
    }

    pub fn receive(&self, fvar: FVarPtr) -> CellPtr {
        let var = &self.0[fvar.get_index()];
        // var.receive()
        CellPtr::new(0, super::Polarity::Neg) // TODO fix me
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
}

impl<S> BVar<S> {
    pub fn get_or_init(&mut self, this_cell: CellPtr) -> Option<CellPtr> {
        // let that_cell = self.0.get_or_init(||{this_cell}).await; // TODO we dont need this
        // if that_cell.get_polarity() != this_cell.get_polarity() {
        //     Some(*that_cell)
        // }
        // else {
            None
        // }
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
pub struct BVars<S : Default = BoundStore>(Vec<BVar<S>>);
impl<S: Default> BVars<S> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn add(&mut self, bvar: BVar<S>) -> BVarPtr {
        let index = self.0.len();
        self.0.push(bvar);
        BVarPtr::new(index)
    }

    pub fn iter(&self) -> ArenaIter<BVar<S>,BVarPtr> {
        ArenaIter::new(&self.0)
    }

    pub fn add_all(&mut self, bvars: BVars<S>) {
        self.0.extend(bvars.0)
    }
}


#[derive(Clone,Copy)]
pub struct VarPtr(u32);
impl VarPtr {
    const INDEX   : BitSet32 = BitSet32 { mask: 0b00111111_11111111_11111111_11111111, offset: 0 };
    const FREE    : BitSet32 = BitSet32 { mask: 0b01, offset: 30 };
    const _UNUSED : BitSet32 = BitSet32 { mask: 0b1, offset: 31 };

    fn new(index: usize, free: bool) -> Self {
        assert!(index < (u32::MAX - 1) as usize);
        let mut var = Self(index as u32);
        var.set_free(free);
        var.set_index(index);
        var
    }

    #[inline]
    pub fn get_raw(&self) -> u32 {
        self.0
    }

    #[inline]
    pub fn is_free(&self) -> bool {
        Self::FREE.get(self.0) > 0
    }

    #[inline]
    fn set_free(&mut self, free: bool) {
        self.0 = Self::FREE.set(self.0, free as u32)
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
        let mut b = f.debug_struct("VarPtr");
        b.field("free", &self.is_free());
        b.field("index", &self.get_index());
        b.finish()
    }
}

impl Into<BVarPtr> for VarPtr {
    fn into(self) -> BVarPtr {
        match self.is_free() {
            true => panic!(),
            false => BVarPtr(self),
        }
    }
}

impl Into<FVarPtr> for VarPtr {
    fn into(self) -> FVarPtr {
        match self.is_free() {
            true => FVarPtr(self),
            false => panic!(),
        }
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
            super::cell::PortKind::FVar => VarPtr(value.get_raw()),
            super::cell::PortKind::BVar => VarPtr(value.get_raw()),
            _ => panic!()
        }
    }
}

#[derive(Debug,Clone,Copy)]
pub struct BVarPtr(VarPtr);
impl BVarPtr {
    pub fn new(index:usize) -> Self {
        Self(VarPtr::new(index, false))
    }

    pub fn get_index(&self) -> usize {
        self.0.get_index()
    }

    pub fn get_raw(&self) -> u32 {
        self.0.get_raw()
    }
}

impl Into<VarPtr> for BVarPtr {
    fn into(self) -> VarPtr {
        VarPtr::new(self.get_index(), false)
    }
}

impl Into<PortPtr> for BVarPtr {
    fn into(self) -> PortPtr {
        PortPtr::new_bvar(self)
    }
}

impl From<PortPtr> for BVarPtr {
    fn from(value: PortPtr) -> Self {
        match value.get_kind() {
            super::cell::PortKind::BVar => BVarPtr(VarPtr::from(value)),
            _ => panic!()
        }
    }
}




#[derive(Debug,Clone,Copy)]
pub struct FVarPtr(VarPtr);
impl FVarPtr {
    pub fn new(index:usize) -> Self {
        Self(VarPtr::new(index, true))
    }

    pub fn get_index(&self) -> usize {
        self.0.get_index()
    }

    pub fn get_raw(&self) -> u32 {
        self.0.get_raw()
    }
}

impl Into<VarPtr> for FVarPtr {
    fn into(self) -> VarPtr {
        VarPtr::new(self.get_index(), true)
    }
}

impl Into<PortPtr> for FVarPtr {
    fn into(self) -> PortPtr {
        PortPtr::new_fvar(self)
    }
}

impl From<PortPtr> for FVarPtr {
    fn from(value: PortPtr) -> Self {
        match value.get_kind() {
            super::cell::PortKind::FVar => FVarPtr(VarPtr::from(value)),
            _ => panic!()
        }
    }
}