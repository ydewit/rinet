use std::{
    fmt::{Binary, Debug, Formatter},
    marker::PhantomData,
};

use super::{
    arena::{Arena, ArenaPtr, ArenaValue},
    symbol::SymbolPtr,
    term::{TermFamily, TermPtr},
    BitSet32, BitSet64, Polarity,
};

#[derive(Clone, Copy, PartialEq)]
pub struct CellPtr(u32);
impl CellPtr {
    const INDEX: BitSet32<23> = BitSet32 {
        mask: 0b00000000_01111111_11111111_11111111,
        offset: 0,
    };
    const POLARITY: BitSet32<1> = BitSet32 {
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

    pub fn new(index: usize, polarity: Polarity) -> Self {
        let mut new = Self(0);
        // new.set_kind(TermKind::Cell);
        new.set_polarity(polarity);
        new.set_index(index);
        new
    }

    #[inline]
    pub fn get_ptr(&self) -> u32 {
        Self::PTR.get(self.0)
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

impl ArenaPtr for CellPtr {
    fn get_index(&self) -> usize {
        self.get_index()
    }
}

impl Binary for CellPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:09b}_{:01b}_{:023b}",
            Self::_UNUSED.get(self.0),
            self.get_polarity() as u8,
            self.get_index()
        )
    }
}

impl Into<TermPtr> for CellPtr {
    fn into(self) -> TermPtr {
        TermPtr::new_cell(self)
    }
}

impl From<u32> for CellPtr {
    fn from(raw: u32) -> Self {
        Self(raw)
    }
}

impl From<TermPtr> for CellPtr {
    fn from(value: TermPtr) -> Self {
        CellPtr(value.get_term())
    }
}

impl Debug for CellPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("CellPtr({:032b})", self.0);
        let mut b = f.debug_struct(&name);
        b.field("polarity", &self.get_polarity());
        b.field("index", &self.get_index());
        b.finish()
    }
}

#[derive(Clone, Copy)]
pub struct Cell<T: TermFamily>(u64, PhantomData<T>);
impl<T: TermFamily> Cell<T> {
    const RIGHT_PORT: BitSet64<25> = BitSet64 {
        mask: 0b00000000_00000000_00000000_00000000_00000001_11111111_11111111_11111111,
        offset: 0,
    };
    const LEFT_PORT: BitSet64<25> = BitSet64 {
        mask: 0b00000000_00000011_11111111_11111111_1111111,
        offset: 25,
    };
    const SYMBOL: BitSet64<14> = BitSet64 {
        mask: 0b11111111_111111,
        offset: 50,
    };

    pub fn new0(symbol_ptr: SymbolPtr) -> Self {
        let mut cell = Self(0, PhantomData);
        cell.set_symbol_ptr(symbol_ptr);
        cell
    }

    pub fn new1(symbol_ptr: SymbolPtr, port: TermPtr) -> Self {
        let mut cell = Self(0, PhantomData);
        cell.set_symbol_ptr(symbol_ptr);
        cell.set_left_port(port);
        cell
    }

    pub fn new2(symbol_ptr: SymbolPtr, left_port: TermPtr, right_port: TermPtr) -> Self {
        let mut cell = Self(0, PhantomData);
        cell.set_symbol_ptr(symbol_ptr);
        cell.set_left_port(left_port);
        cell.set_right_port(right_port);
        cell
    }

    #[inline]
    pub fn get_symbol_ptr(&self) -> SymbolPtr {
        SymbolPtr::from(Self::SYMBOL.get(self.0))
    }

    #[inline]
    fn set_symbol_ptr(&mut self, symbol_ptr: SymbolPtr) {
        self.0 = Self::SYMBOL.set(self.0, symbol_ptr.into())
    }

    #[inline]
    pub fn get_left_port(&self) -> TermPtr {
        (Self::LEFT_PORT.get(self.0) as u32).into()
    }

    #[inline]
    fn set_left_port(&mut self, port: TermPtr) {
        self.0 = Self::LEFT_PORT.set(self.0, port.get_ptr() as u64)
    }

    #[inline]
    pub fn get_right_port(&self) -> TermPtr {
        (Self::RIGHT_PORT.get(self.0) as u32).into()
    }

    fn set_right_port(&mut self, port: TermPtr) {
        self.0 = Self::RIGHT_PORT.set(self.0, port.get_ptr() as u64)
    }

    pub fn to_ptr(&self, index: usize) -> CellPtr {
        CellPtr::new(index, self.get_symbol_ptr().get_polarity())
    }
}

impl<T: TermFamily> ArenaValue<CellPtr> for Cell<T> {
    fn to_ptr(&self, index: usize) -> CellPtr {
        CellPtr::new(index, self.get_symbol_ptr().get_polarity())
    }
}

impl<T: TermFamily> Binary for Cell<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:014b}_{:025b}_{:025b}",
            self.get_symbol_ptr().get_raw(),
            self.get_left_port().get_ptr(),
            self.get_right_port().get_ptr()
        )
    }
}

impl<T: TermFamily> Debug for Cell<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("Cell({:064b})", self.0);
        let mut b = f.debug_struct(&name);
        b.field("symbol", &self.get_symbol_ptr());
        b.field("left", &self.get_left_port());
        b.field("right", &self.get_right_port());
        b.finish()
    }
}

pub type Cells<T: TermFamily> = Arena<Cell<T>, CellPtr>;

#[cfg(test)]
mod tests {

    use super::*;
}
