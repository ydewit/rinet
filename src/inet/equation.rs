use std::{
    fmt::{Binary, Debug, Display, Formatter},
    marker::PhantomData,
};

use crate::inet::Polarity;

use super::{
    arena::{ArenaIter, ToPtr},
    cell::{CellItem, CellPtr, Cells},
    symbol::SymbolBook,
    term::TermFamily,
    var::{VarItem, VarPtr, Vars},
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

impl<T: TermFamily> ToPtr<EquationPtr> for Equation<T> {
    fn to_ptr(&self, index: usize) -> EquationPtr {
        EquationPtr::new(index, self.get_kind())
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

pub struct EquationItem<'a, T: TermFamily> {
    pub ptr: EquationPtr,
    pub symbols: &'a SymbolBook,
    pub equations: &'a Equations<T>,
    pub cells: &'a Cells<T>,
    pub vars: &'a Vars<T>,
}
impl<'a, T: TermFamily> EquationItem<'a, T> {
    fn to_cell_item(&self, cell_ptr: CellPtr) -> CellItem<'a, T> {
        CellItem {
            cell_ptr,
            symbols: self.symbols,
            cells: self.cells,
            vars: self.vars,
        }
    }

    fn to_var_item(&self, var_ptr: VarPtr) -> VarItem<'a, T> {
        VarItem {
            var_ptr,
            vars: self.vars,
        }
    }
}
impl<'a, T: TermFamily> Display for EquationItem<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let eqn = self.equations.get(self.ptr);
        match eqn.get_kind() {
            EquationKind::Redex => {
                write!(
                    f,
                    "{} = {}",
                    self.to_cell_item(eqn.get_redex_ctr()),
                    self.to_cell_item(eqn.get_redex_fun())
                )
            }
            EquationKind::Bind => {
                write!(
                    f,
                    "{} ← {}",
                    self.to_var_item(eqn.get_bind_var()),
                    self.to_cell_item(eqn.get_bind_cell())
                )
            }
            EquationKind::Connect => {
                write!(
                    f,
                    "{} ↔ {}",
                    self.to_var_item(eqn.get_connect_left()),
                    self.to_var_item(eqn.get_connect_right())
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct Equations<T: TermFamily>(Vec<Equation<T>>, PhantomData<T>);
impl<T: TermFamily> Equations<T> {
    pub fn new() -> Self {
        Self(Vec::new(), PhantomData)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity), PhantomData)
    }

    pub fn get(&self, equation: EquationPtr) -> Equation<T> {
        self.0[equation.get_index()].clone()
    }

    pub fn redex(&mut self, ctr: CellPtr, fun: CellPtr) -> EquationPtr {
        self.add(Equation::redex(ctr, fun))
    }

    pub fn reuse_redex(&mut self, ptr: EquationPtr, ctr: CellPtr, fun: CellPtr) {
        self.0[ptr.get_index()] = Equation::redex(ctr, fun);
    }

    pub fn bind(&mut self, var: VarPtr, cell: CellPtr) -> EquationPtr {
        self.add(Equation::bind(var, cell))
    }

    pub fn reuse_bind(&mut self, ptr: EquationPtr, var: VarPtr, cell: CellPtr) {
        self.0[ptr.get_index()] = Equation::bind(var, cell);
    }

    pub fn connect(&mut self, left: VarPtr, right: VarPtr) -> EquationPtr {
        self.add(Equation::connect(left, right))
    }

    pub fn reuse_connect(&mut self, ptr: EquationPtr, left: VarPtr, right: VarPtr) {
        self.0[ptr.get_index()] = Equation::connect(left, right);
    }

    pub fn add_all(&mut self, equations: Equations<T>) {
        self.0.extend(equations.0)
    }

    pub fn all(&mut self) -> std::vec::Drain<Equation<T>> {
        self.0.drain(..)
    }

    fn add(&mut self, equation: Equation<T>) -> EquationPtr {
        let index = self.0.len();
        let ptr = equation.to_ptr(index);
        self.0.push(equation);
        ptr
    }

    pub fn iter(&self) -> ArenaIter<Equation<T>, EquationPtr> {
        ArenaIter::new(&self.0)
    }
}

pub struct EquationsItem<'a, T: TermFamily> {
    pub symbols: &'a SymbolBook,
    pub equations: &'a Equations<T>,
    pub cells: &'a Cells<T>,
    pub vars: &'a Vars<T>,
}
impl<'a, T: TermFamily> EquationsItem<'a, T> {
    fn to_equation_item(&self, eqn_ptr: EquationPtr) -> EquationItem<T> {
        EquationItem {
            ptr: eqn_ptr,
            symbols: self.symbols,
            equations: self.equations,
            cells: self.cells,
            vars: self.vars,
        }
    }
}
impl<'a, T: TermFamily> Display for EquationsItem<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.equations.iter().fold(Ok(()), |result, eqn_ptr| {
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
