use core::panic;
use std::fmt::{Formatter, Debug};

use super::{symbol::SymbolPtr, Polarity, var::{FVarPtr, BVarPtr}, arena::{ToPtr, ArenaIter}, BitSet32, BitSet64};


#[derive(Debug, PartialEq)]
pub enum PortKind {
    Cell = 0,
    FVar = 1,
    BVar = 2
}
impl PortKind {
    pub const MAX : u8 = 0b11;
}

impl From<u64> for PortKind {
    fn from(value: u64) -> Self {
        match value {
            0 => PortKind::Cell,
            1 => PortKind::FVar,
            2 => PortKind::BVar,
            _ => panic!()
        }
    }
}

impl From<u32> for PortKind {
    fn from(value: u32) -> Self {
        match value {
            0 => PortKind::Cell,
            1 => PortKind::FVar,
            2 => PortKind::BVar,
            _ => panic!()
        }
    }
}


#[derive(Clone,Copy)]
pub struct PortPtr(u32);
impl PortPtr {
    const PTR    : BitSet32 = BitSet32{ mask: 0b00000000_11111111_11111111_11111111, offset: 0 };
    const KIND   : BitSet32 = BitSet32{ mask: 0b00000001, offset: 24 };
    const _UNUSED : BitSet32 = BitSet32{ mask: 0b11111110, offset: 25 };

    pub fn new_fvar(ptr: FVarPtr) -> Self {
        let mut this = Self(ptr.get_raw());
        this.set_kind(PortKind::FVar);
        this
    }

    pub fn new_bvar(ptr: BVarPtr) -> Self {
        let mut this = Self(ptr.get_raw());
        this.set_kind(PortKind::BVar);
        this
    }

    pub fn new_cell(ptr: CellPtr) -> Self {
        let mut this = Self(ptr.get_raw());
        this.set_kind(PortKind::Cell);
        this
    }

    fn set_kind(&mut self, kind: PortKind) {
        self.0 = Self::KIND.set(self.0, kind as u32)
    }

    pub fn get_kind(&self) -> PortKind {
        PortKind::from(Self::KIND.get(self.0))
    }

    pub fn get_fvar(&self) -> FVarPtr {
        assert!(self.get_kind() == PortKind::FVar);
        FVarPtr::from(*self)
    }

    pub fn get_bvar(&self) -> BVarPtr {
        assert!(self.get_kind() == PortKind::BVar);
        BVarPtr::from(*self)
    }

    pub fn get_cell(&self) -> CellPtr {
        assert!(self.get_kind() == PortKind::Cell);
        CellPtr::from(*self)
    }

    fn get_ptr(&self) -> u32 {
        Self::PTR.get(self.0)
    }

    fn set_ptr(&mut self, raw_ptr: u32) {
        self.0 = Self::PTR.set(self.0, raw_ptr)
    }

    pub fn get_raw(self) -> u32 {
        self.0
    }
}

impl From<u64> for PortPtr {
    fn from(value: u64) -> Self {
        assert!(value <= 1 << 25);
        Self(value as u32)
    }
}

impl Debug for PortPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut b = f.debug_struct("PortPtr");
        b.field("kind", &self.get_kind());
        match self.get_kind() {
            PortKind::Cell => b.field("cell", &self.get_cell()),
            PortKind::FVar => b.field("fvar", &self.get_fvar()),
            PortKind::BVar => b.field("bvar", &self.get_bvar())
        };
        b.finish()
    }
}

#[derive(Clone,Copy)]
pub struct CellPtr(u32);
impl CellPtr {
    const INDEX    : BitSet32 = BitSet32{ mask: 0b00000000_01111111_11111111_11111111, offset: 0 };
    const POLARITY : BitSet32 = BitSet32{ mask: 0b00000000_1, offset: 23 };
    const _UNUSED  : BitSet32 = BitSet32{ mask: 0b11111111, offset: 24 };

    pub fn new(index: usize, polarity: Polarity) -> Self {
        let mut new = Self(0);
        // new.set_kind(PortKind::Cell);
        new.set_polarity(polarity);
        new.set_index(index);
        new
    }

    #[inline]
    pub fn get_raw(&self) -> u32 {
        self.0
    }

    #[inline]
    pub fn get_polarity(&self) -> Polarity {
        Polarity::from(Self::POLARITY.get(self.0))
    }

    #[inline]
    fn set_polarity(&mut self, polarity: Polarity) {
        self.0 = Self::POLARITY.set(self.0, polarity as u32)
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

// impl Future for CellPtr {
//     type Output = CellPtr;

//     fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
//         Poll::Ready(self.to_owned())
//     }
// }
impl Into<PortPtr> for CellPtr {
    fn into(self) -> PortPtr {
        PortPtr::new_cell(self)
    }
}

impl From<u32> for CellPtr {
    fn from(raw: u32) -> Self {
        Self(raw)
    }
}

impl From<PortPtr> for CellPtr {
    fn from(value: PortPtr) -> Self {
        CellPtr(value.get_raw())
    }
}

impl Debug for CellPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut b = f.debug_struct("CellPtr");
        b.field("polarity", &self.get_polarity());
        b.field("index", &self.get_index());
        b.finish()
    }
}

#[derive(Clone,Copy)]
pub struct Cell(u64);
impl Cell {
    const RIGHT_PORT  : BitSet64 = BitSet64{ mask: 0b00000000_00000000_00000000_00000000_00000000_11111111_11111111_11111111, offset: 0 };
    const LEFT_PORT   : BitSet64 = BitSet64{ mask: 0b00000000_00000000_11111111_11111111_11111111, offset: 24 };
    // const RIGHT_KIND  : BitSet64 = BitSet64{ mask: 0b00000000_00000001, offset: 48 };
    // const LEFT_KIND   : BitSet64 = BitSet64{ mask: 0b00000000_0000001, offset: 49 };
    const SYMBOL      : BitSet64 = BitSet64{ mask: 0b11111111_111111, offset: 50 };

    pub fn new0(symbol: SymbolPtr) -> Self {
        let mut cell = Self(0);
        cell.set_symbol(symbol);
        cell
    }

    pub fn new1(symbol: SymbolPtr, left_port: PortPtr) -> Self {
        let mut cell = Self(0);
        cell.set_symbol(symbol);
        cell.set_left_port(left_port);
        cell
    }

    pub fn new2(symbol: SymbolPtr, left_port: PortPtr, right_port: PortPtr) -> Self {
        let mut cell = Self(0);
        cell.set_symbol(symbol);
        cell.set_left_port(left_port);
        cell.set_right_port(right_port);
        cell
    }

    #[inline]
    pub fn get_symbol(&self) -> SymbolPtr {
        SymbolPtr::from(Self::SYMBOL.get(self.0))
    }

    #[inline]
    fn set_symbol(&mut self, symbol: SymbolPtr) {
        self.0 = Self::SYMBOL.set(self.0, symbol.get_raw() as u64)
    }

    // #[inline]
    // pub fn get_left_kind(&self) -> PortKind {
    //     PortKind::from(Self::LEFT_KIND.get(self.0))
    // }

    // #[inline]
    // fn set_left_kind(&mut self, kind: PortKind) {
    //     self.0 = Self::LEFT_KIND.set(self.0, kind as u64)
    // }

    #[inline]
    pub fn get_left_port(&self) -> PortPtr {
        PortPtr::from(Self::LEFT_PORT.get(self.0))
    }

    #[inline]
    fn set_left_port(&mut self, port: PortPtr) {
        self.0 = Self::LEFT_PORT.set(self.0, port.get_ptr() as u64)
    }

    // #[inline]
    // pub fn get_right_kind(&self) -> PortKind {
    //     PortKind::from(Self::RIGHT_KIND.get(self.0))
    // }

    // #[inline]
    // fn set_right_kind(&mut self, kind: PortKind) {
    //     self.0 = Self::RIGHT_KIND.set(self.0, kind as u64)
    // }

    #[inline]
    pub fn get_right_port(&self) -> PortPtr {
        PortPtr::from(Self::RIGHT_PORT.get(self.0))
    }

    fn set_right_port(&mut self, port: PortPtr) {
        self.0 = Self::RIGHT_PORT.set(self.0, port.get_ptr() as u64)
    }

    pub fn to_ptr(&self, index: usize) -> CellPtr {
        CellPtr::new(index, self.get_symbol().get_polarity())
    }
}

impl ToPtr<CellPtr> for Cell {
    fn to_ptr(&self, index: usize) -> CellPtr {
        CellPtr::new(index, self.get_symbol().get_polarity())
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut b = f.debug_struct("Cell");
        b.field("symbol", &self.get_symbol());
        b.field("left", &self.get_left_port());
        b.field("right", &self.get_right_port());
        b.finish()
    }
}

#[derive(Debug)]
pub struct Cells(Vec<Cell>);
impl Cells {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn iter(&self) -> ArenaIter<Cell,CellPtr> {
        ArenaIter::new(&self.0)
    }

    pub fn get(&self, cell: CellPtr) -> Cell {
        self.0[cell.get_index()]
    }

    pub fn add_all(&mut self, cells: Cells) {
        self.0.extend(cells.0)
    }

    pub fn add(&mut self, cell: Cell) -> CellPtr {
        let index = self.0.len();
        self.0.push(cell);
        cell.to_ptr(index)
    }
}


#[cfg(test)]
mod tests {
    use crate::inet::symbol::Symbol;

    use super::*;

    #[test]
    fn test_cell0_neg() {
        let symbol_pos = SymbolPtr::new(2, Polarity::Pos);
        let mut cell = Cell::new0(symbol_pos);
        assert_eq!(cell.get_symbol(), symbol_pos);
        let symbol_neg = SymbolPtr::new(2, Polarity::Pos);
        cell.set_symbol(symbol_neg);
        assert_eq!(cell.get_symbol(), symbol_neg);
    }

}