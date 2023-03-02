use std::{
    fmt::{Binary, Debug, Display, Formatter},
    marker::PhantomData,
};

use crate::inet::Polarity;

use super::{
    arena::{Arena, ArenaPtr, ArenaValue},
    cell::CellPtr,
    heap::Heap,
    symbol::SymbolBook,
    term::TermFamily,
    var::VarPtr,
    BitSet16, BitSet64,
};

#[derive(Debug, PartialEq)]
pub enum EquationKind {
    Redex = 0,
    Bind = 1,
    Connect = 2,
}

impl From<u8> for EquationKind {
    fn from(value: u8) -> Self {
        if value == 0 {
            EquationKind::Redex
        } else if value == 1 {
            EquationKind::Bind
        } else if value == 2 {
            EquationKind::Connect
        } else {
            panic!()
        }
    }
}

impl From<u16> for EquationKind {
    fn from(value: u16) -> Self {
        if value == 0 {
            EquationKind::Redex
        } else if value == 1 {
            EquationKind::Bind
        } else if value == 2 {
            EquationKind::Connect
        } else {
            panic!()
        }
    }
}

impl From<u64> for EquationKind {
    fn from(value: u64) -> Self {
        if value == 0 {
            EquationKind::Redex
        } else if value == 1 {
            EquationKind::Bind
        } else if value == 2 {
            EquationKind::Connect
        } else {
            panic!()
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub struct EquationPtr(u16);
impl EquationPtr {
    const INDEX: BitSet16<14> = BitSet16 {
        mask: 0b00111111_11111111,
        offset: 0,
    };
    const KIND: BitSet16<2> = BitSet16 {
        mask: 0b11,
        offset: 14,
    };

    pub fn new(index: usize, kind: EquationKind) -> Self {
        let mut eqn = Self(0);
        eqn.set_kind(kind);
        eqn.set_index(index);
        eqn
    }

    #[inline]
    pub fn get_kind(&self) -> EquationKind {
        EquationKind::from(Self::KIND.get(self.0))
    }

    #[inline]
    fn set_kind(&mut self, kind: EquationKind) {
        self.0 |= Self::KIND.set(self.0, kind as u16)
    }

    #[inline]
    pub fn get_index(&self) -> usize {
        Self::INDEX.get(self.0) as usize
    }

    fn set_index(&mut self, index: usize) {
        assert!(index < (u16::MAX - Self::KIND.mask as u16) as usize);
        self.0 |= Self::INDEX.set(self.0, index as u16)
    }
}

impl ArenaPtr for EquationPtr {
    fn get_index(&self) -> usize {
        self.get_index()
    }
}

impl Binary for EquationPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Binary::fmt(&self.0, f)
    }
}

impl Debug for EquationPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("EquationPtr({:016b})", self.0);
        let mut b = f.debug_struct(&name);
        b.field("kind", &self.get_kind());
        b.field("index", &self.get_index());
        b.finish()
    }
}

#[derive(Clone, Copy)]
pub struct Equation<T: TermFamily>(pub u64, PhantomData<T>);
impl<T: TermFamily> Equation<T> {
    const RIGHT: BitSet64<31> = BitSet64 {
        mask: 0b00000000_00000000_00000000_00000000_01111111_11111111_11111111_11111111,
        offset: 0,
    };
    const LEFT: BitSet64<31> = BitSet64 {
        mask: 0b00111111_11111111_11111111_11111111_1,
        offset: 31,
    };
    const KIND: BitSet64<2> = BitSet64 {
        mask: 0b11,
        offset: 62,
    };

    pub fn redex(left: CellPtr, right: CellPtr) -> Self {
        assert!(left.get_polarity() == Polarity::Pos && right.get_polarity() == Polarity::Neg);
        let mut eqn = Equation(0, PhantomData);
        eqn.reuse_redex(left, right);
        eqn
    }

    pub fn bind(var: VarPtr, cell: CellPtr) -> Self {
        let mut eqn = Equation(0, PhantomData);
        eqn.reuse_bind(var, cell);
        eqn
    }

    pub fn connect(left: VarPtr, right: VarPtr) -> Self {
        let mut eqn = Equation(0, PhantomData);
        eqn.reuse_connect(left, right);
        eqn
    }

    pub fn reuse_redex(&mut self, left: CellPtr, right: CellPtr) {
        self.set_kind(EquationKind::Redex);
        self.set_left(left.get_ptr());
        self.set_right(right.get_ptr());
    }

    pub fn reuse_bind(&mut self, var: VarPtr, cell: CellPtr) {
        self.set_kind(EquationKind::Bind);
        self.set_left(var.get_ptr());
        self.set_right(cell.get_ptr());
    }

    pub fn reuse_connect(&mut self, left: VarPtr, right: VarPtr) {
        self.set_kind(EquationKind::Connect);
        self.set_left(left.get_ptr());
        self.set_right(right.get_ptr());
    }

    #[inline]
    pub fn get_kind(&self) -> EquationKind {
        EquationKind::from(Self::KIND.get(self.0))
    }

    #[inline]
    fn set_kind(&mut self, kind: EquationKind) {
        self.0 = Self::KIND.set(self.0, kind as u64)
    }

    #[inline]
    fn get_left(&self) -> u32 {
        Self::LEFT.get(self.0) as u32
    }

    #[inline]
    fn set_left(&mut self, value: u32) {
        self.0 = Self::LEFT.set(self.0, value as u64)
    }

    #[inline]
    fn get_right(&self) -> u32 {
        Self::RIGHT.get(self.0) as u32
    }

    #[inline]
    fn set_right(&mut self, value: u32) {
        self.0 = Self::RIGHT.set(self.0, value as u64)
    }

    #[inline]
    pub fn to_ptr(&self, index: usize) -> EquationPtr {
        EquationPtr::new(index, self.get_kind())
    }

    #[inline]
    pub fn get_redex_ctr(&self) -> CellPtr {
        assert!(self.get_kind() == EquationKind::Redex);
        CellPtr::from(self.get_left())
    }

    #[inline]
    pub fn get_redex_fun(&self) -> CellPtr {
        assert!(self.get_kind() == EquationKind::Redex);
        CellPtr::from(self.get_right())
    }

    #[inline]
    pub fn get_bind_var(&self) -> VarPtr {
        assert!(self.get_kind() == EquationKind::Bind);
        VarPtr::from(self.get_left())
    }

    #[inline]
    pub fn get_bind_cell(&self) -> CellPtr {
        CellPtr::from(self.get_right())
    }

    #[inline]
    pub fn get_connect_left(&self) -> VarPtr {
        VarPtr::from(self.get_left())
    }

    #[inline]
    pub fn get_connect_right(&self) -> VarPtr {
        VarPtr::from(self.get_right())
    }
}

impl<T: TermFamily> ArenaValue<EquationPtr> for Equation<T> {
    fn to_ptr(&self, index: usize) -> EquationPtr {
        EquationPtr::new(index, self.get_kind())
    }
}

impl<T: TermFamily> Binary for Equation<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02b}_{:031b}_{:031b}",
            self.get_kind() as u8,
            self.get_left(),
            self.get_right()
        )
    }
}

impl<T: TermFamily> Debug for Equation<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("Equation({:064b})", self.0);
        let mut b = f.debug_struct(&name);
        b.field("kind", &self.get_kind());
        match self.get_kind() {
            EquationKind::Redex => {
                b.field("left", &self.get_redex_ctr());
                b.field("right", &self.get_redex_fun());
            }
            EquationKind::Bind => {
                b.field("var", &self.get_bind_var());
                b.field("cell", &self.get_bind_cell());
            }
            EquationKind::Connect => {
                b.field("left", &self.get_connect_left());
                b.field("right", &self.get_connect_right());
            }
        }
        b.finish()
    }
}

pub struct EquationDisplay<'a, T: TermFamily> {
    pub ptr: EquationPtr,
    pub symbols: &'a SymbolBook,
    pub body: &'a Equations<T>,
    pub heap: &'a Heap<T>,
}
impl<'a, T: TermFamily> Display for EquationDisplay<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let eqn = self.body.get(self.ptr).unwrap();
        match eqn.get_kind() {
            EquationKind::Redex => {
                write!(
                    f,
                    "{} = {}",
                    self.heap.display_cell(self.symbols, eqn.get_redex_ctr()),
                    self.heap.display_cell(self.symbols, eqn.get_redex_fun())
                )
            }
            EquationKind::Bind => {
                write!(
                    f,
                    "{} ← {}",
                    self.heap.display_var(self.symbols, eqn.get_bind_var()),
                    self.heap.display_cell(self.symbols, eqn.get_bind_cell())
                )
            }
            EquationKind::Connect => {
                write!(
                    f,
                    "{} ↔ {}",
                    self.heap.display_var(self.symbols, eqn.get_connect_left()),
                    self.heap.display_var(self.symbols, eqn.get_connect_right())
                )
            }
        }
    }
}

pub type Equations<T: TermFamily> = Arena<Equation<T>, EquationPtr>;

pub struct EquationsDisplay<'a, T: TermFamily> {
    pub symbols: &'a SymbolBook,
    pub body: &'a Equations<T>,
    pub heap: &'a Heap<T>,
}
impl<'a, T: TermFamily> EquationsDisplay<'a, T> {
    fn to_equation_item(&self, eqn_ptr: EquationPtr) -> EquationDisplay<T> {
        EquationDisplay {
            ptr: eqn_ptr,
            symbols: self.symbols,
            body: self.body,
            heap: self.heap,
        }
    }
}
impl<'a, T: TermFamily> Display for EquationsDisplay<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.body.iter().fold(Ok(()), |result, eqn_ptr| {
            result.and_then(|_| write!(f, " {}", self.to_equation_item(eqn_ptr)))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equation_kind_from_u8() {
        assert_eq!(EquationKind::from(0_u8), EquationKind::Redex);
        assert_eq!(EquationKind::from(1_u8), EquationKind::Bind);
        assert_eq!(EquationKind::from(2_u8), EquationKind::Connect);
        assert!(std::panic::catch_unwind(|| EquationKind::from(3_u8)).is_err());
    }

    #[test]
    fn test_equation_kind_from_u16() {
        assert_eq!(EquationKind::from(0_u16), EquationKind::Redex);
        assert_eq!(EquationKind::from(1_u16), EquationKind::Bind);
        assert_eq!(EquationKind::from(2_u16), EquationKind::Connect);
        assert!(std::panic::catch_unwind(|| EquationKind::from(3_u16)).is_err());
    }

    #[test]
    fn test_equation_kind_from_u64() {
        assert_eq!(EquationKind::from(0_u64), EquationKind::Redex);
        assert_eq!(EquationKind::from(1_u64), EquationKind::Bind);
        assert_eq!(EquationKind::from(2_u64), EquationKind::Connect);
        assert!(std::panic::catch_unwind(|| EquationKind::from(3_u64)).is_err());
    }

    #[test]
    fn test_equation_ptr_new() {
        let eqn_ptr = EquationPtr::new(100, EquationKind::Redex);
        assert_eq!(eqn_ptr.get_index(), 100);
        assert_eq!(eqn_ptr.get_kind(), EquationKind::Redex);
    }

    #[test]
    fn test_equation_ptr_set_and_get_index() {
        let mut eqn_ptr = EquationPtr::new(100, EquationKind::Redex);
        assert_eq!(eqn_ptr.get_index(), 100);

        eqn_ptr.set_index(200);
        assert_eq!(eqn_ptr.get_index(), 200);
    }

    #[test]
    fn test_equation_ptr_set_and_get_kind() {
        let mut eqn_ptr = EquationPtr::new(100, EquationKind::Redex);
        assert_eq!(eqn_ptr.get_kind(), EquationKind::Redex);

        eqn_ptr.set_kind(EquationKind::Bind);
        assert_eq!(eqn_ptr.get_kind(), EquationKind::Bind);
    }

    #[test]
    fn test_equation_binary_fmt() {
        let eqn_ptr = EquationPtr::new(100, EquationKind::Redex);
        assert_eq!(format!("{:b}", eqn_ptr), "1010100000000110");
    }

    #[test]
    fn test_equation_debug_fmt() {
        let eqn_ptr = EquationPtr::new(100, EquationKind::Redex);
        assert_eq!(
            format!("{:?}", eqn_ptr),
            "EquationPtr { kind: Redex, index: 100 }"
        );
    }
}
