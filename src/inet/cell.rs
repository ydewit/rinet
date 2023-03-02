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

    // // #[test]
    // // fn test_cell0_neg() {
    // //     let symbol_pos = SymbolPtr::new(2, SymbolArity::One, Polarity::Pos);
    // //     let mut cell = Cell::new0(symbol_pos);
    // //     assert_eq!(cell.get_symbol_ptr(), symbol_pos);
    // //     let symbol_neg = SymbolPtr::new(2, SymbolArity::One, Polarity::Pos);
    // //     cell.set_symbol_ptr(symbol_neg);
    // //     assert_eq!(cell.get_symbol_ptr(), symbol_neg);
    // // }

    // #[test]
    // fn port_kind_from_u64() {
    //     assert_eq!(TermKind::Cell, TermKind::from(0u64));
    //     assert_eq!(TermKind::Var, TermKind::from(1u64));
    //     assert!(matches!(TermKind::from(0u64), TermKind::Cell));
    //     assert!(matches!(TermKind::from(1u64), TermKind::Var));
    // }

    // #[test]
    // fn port_kind_from_u32() {
    //     assert_eq!(TermKind::Cell, TermKind::from(0u32));
    //     assert_eq!(TermKind::Var, TermKind::from(1u32));
    //     assert!(matches!(TermKind::from(0u32), TermKind::Cell));
    //     assert!(matches!(TermKind::from(1u32), TermKind::Var));
    // }

    // #[test]
    // fn port_ptr_get_kind() {
    //     let fvar = VarPtr::new(42);
    //     println!("{}", fvar.get_index());
    //     // println!("{}", fvar.get_ptr());
    //     println!("{:0b}", 8388650);
    //     println!("{:0b}", 25165866);

    //     let bvar = VarPtr::new(42);
    //     let p1 = TermPtr::new_var(fvar.into());
    //     // println!("{}", p1.get_kind());
    //     // println!("{}", fvar.get_ptr());
    //     let p2 = TermPtr::new_var(bvar.into());
    //     let p3 = TermPtr::new_cell(CellPtr::new(42, Polarity::Pos));

    //     assert_eq!(TermKind::Var, p1.get_kind());
    //     assert_eq!(fvar, p1.into());
    //     assert_eq!(TermKind::Var, p2.get_kind());
    //     assert_eq!(bvar, p2.into());
    //     assert_eq!(TermKind::Cell, p3.get_kind());
    // }

    // #[test]
    // fn port_ptr_get_fvar() {
    //     let fvar = VarPtr::new(42);
    //     let p = TermPtr::new_var(fvar.into());

    //     assert_eq!(fvar, p.get_var().into());
    // }

    // #[test]
    // fn port_ptr_get_bvar() {
    //     let bvar = VarPtr::new(42);
    //     let p = TermPtr::new_var(bvar.into());

    //     assert_eq!(bvar, p.get_var().into());
    // }

    // #[test]
    // fn port_ptr_get_cell() {
    //     let cell = CellPtr::new(42, Polarity::Pos);
    //     let p = TermPtr::new_cell(cell);

    //     assert_eq!(cell, p.get_cell());
    // }

    // #[test]
    // #[should_panic]
    // fn port_ptr_get_fvar_wrong_kind() {
    //     let cell = CellPtr::new(42, Polarity::Pos);
    //     let p = TermPtr::new_cell(cell);
    //     p.get_var();
    // }

    // #[test]
    // #[should_panic]
    // fn port_ptr_get_cell_wrong_kind() {
    //     let fvar = VarPtr::new(42);
    //     let p = TermPtr::new_var(fvar.into());
    //     p.get_cell();
    // }

    // #[test]
    // fn cell_ptr_get_ptr() {
    //     let cell_ptr = CellPtr::new(42, Polarity::Pos);
    //     assert_eq!(42, cell_ptr.get_ptr());
    // }

    // #[test]
    // fn cell_ptr_get_polarity() {
    //     let cell_ptr1 = CellPtr::new(42, Polarity::Pos);
    //     assert_eq!(Polarity::Pos, cell_ptr1.get_polarity());

    //     let cell_ptr2 = CellPtr::new(42, Polarity::Neg);
    //     assert_eq!(Polarity::Neg, cell_ptr2.get_polarity());
    // }

    // // #[test]
    // // fn cell2() {
    // //     let sym_ptr = SymbolPtr::new(3, SymbolArity::One, Polarity::Neg);
    // //     let r = VarPtr::new(5);
    // //     let a: TermPtr = r.into();
    // //     println!("{:0b}", a);
    // //     assert_eq!(TermKind::Var, a.get_kind());
    // //     assert_eq!(a.get_var(), r.into());

    // //     let cell = Cell::new1(sym_ptr, r.into());
    // //     println!("{:0b}", cell);
    // //     assert_eq!(TermKind::Var, cell.get_left_port().get_kind());
    // // }
}
