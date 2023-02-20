use std::fmt::{Debug, Formatter};

use crate::inet::Polarity;

use super::{cell::CellPtr, var::VarPtr, arena::{ToPtr, ArenaIter}, BitSet64};

#[derive(Debug, PartialEq)]
pub enum EquationKind {
    Redex = 0,
    Bind = 1,
    Connect = 2
}

impl EquationKind {
    const MAX: u8 = 0b11;
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

#[derive(Clone,Copy)]
pub struct EquationPtr(u16);
impl EquationPtr {
    pub const KIND_MASK    : u16 = 0b11;

    fn new(index: usize, kind: EquationKind) -> Self {
        assert!(index < (u16::MAX - EquationKind::MAX as u16) as usize);
        let mut eqn = Self(index as u16);
        eqn.set_kind(kind);
        eqn
    }

    #[inline]
    pub fn get_kind(&self) -> EquationKind {
        EquationKind::from(self.0 >> 14 & Self::KIND_MASK)
    }

    #[inline]
    fn set_kind(&mut self, kind: EquationKind) {
        self.0 = self.0 | (kind as u16) << 15;
    }

    #[inline]
    pub fn get_index(&self) -> usize {
        ((self.0 << 2) >> 2) as usize
    }
}

impl Debug for EquationPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut b = f.debug_struct("EquationPtr");
        b.field("kind", &self.get_kind());
        b.field("index", &self.get_index());
        b.finish()
    }
}

#[derive(Clone,Copy)]
pub struct Equation(u64);
impl Equation {
    const RIGHT : BitSet64 = BitSet64{ mask: 0b00000000_00000000_00000000_00000000_01111111_11111111_11111111_11111111, offset: 0 };
    const LEFT  : BitSet64 = BitSet64{ mask: 0b00111111_11111111_11111111_11111111_1, offset: 31 };
    const KIND  : BitSet64 = BitSet64{ mask: 0b11, offset: 62 };

    pub fn redex(left: CellPtr, right: CellPtr) -> Self {
        assert!(left.get_polarity() != right.get_polarity());
        let mut eqn = Equation(0);
        eqn.reuse_redex(left, right);
        eqn
    }

    pub fn bind(var: VarPtr, cell: CellPtr) -> Self {
        let mut eqn = Equation(0);
        eqn.reuse_bind(var, cell);
        eqn
    }

    pub fn connect(left: VarPtr, right: VarPtr) -> Self {
        let mut eqn = Equation(0);
        eqn.reuse_connect(left, right);
        eqn
    }

    pub fn reuse_redex(&mut self, left: CellPtr, right: CellPtr) {
        self.set_kind(EquationKind::Redex);
        self.set_left(left.get_raw());
        self.set_right(right.get_raw());
    }

    pub fn reuse_bind(&mut self, var: VarPtr, cell: CellPtr) {
        self.set_kind(EquationKind::Bind);
        self.set_left(var.get_raw());
        self.set_right(cell.get_raw());
    }

    pub fn reuse_connect(&mut self, left: VarPtr, right: VarPtr) {
        self.set_kind(EquationKind::Connect);
        self.set_left(left.get_raw());
        self.set_right(right.get_raw());
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
    pub fn get_redex_left(&self) -> CellPtr {
        assert!(self.get_kind() == EquationKind::Redex);
        CellPtr::from(self.get_left())
    }

    #[inline]
    pub fn get_redex_right(&self) -> CellPtr {
        assert!(self.get_kind() == EquationKind::Redex);
        CellPtr::from(self.get_right())
    }

    #[inline]
    pub fn get_bind_var(&self) -> VarPtr {
        assert!(self.get_kind() == EquationKind::Bind);
        VarPtr::from(self.get_left())
    }

    #[inline]
    pub fn get_bind_cell(self) -> CellPtr {
        CellPtr::from(self.get_right())
    }

    #[inline]
    pub fn get_connect_left(self) -> VarPtr {
        VarPtr::from(self.get_left())
    }

    #[inline]
    pub fn get_connect_right(self) -> VarPtr {
        VarPtr::from(self.get_right())
    }
}

impl ToPtr<EquationPtr> for Equation {
    fn to_ptr(&self, index: usize) -> EquationPtr {
        EquationPtr::new(index, self.get_kind())
    }
}

impl Debug for Equation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut b = f.debug_struct("Equation");
        b.field("kind", &self.get_kind());
        match self.get_kind() {
            EquationKind::Redex => {
                b.field("left", &self.get_redex_left());
                b.field("right", &self.get_redex_right());
            },
            EquationKind::Bind => {
                b.field("var", &self.get_bind_var());
                b.field("cell", &self.get_bind_cell());
            },
            EquationKind::Connect => {
                b.field("left", &self.get_connect_left());
                b.field("right", &self.get_connect_right());
            },
        }
        b.finish()
    }
}


#[derive(Debug)]
pub struct Equations(Vec<Equation>);
impl Equations {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn get(&self, equation: EquationPtr) -> Equation {
        self.0[equation.get_index()]
    }

    pub fn add_all(&mut self, equations: Equations) {
        self.0.extend(equations.0)
    }

    pub fn add(&mut self, equation: Equation) -> EquationPtr {
        let index = self.0.len();
        self.0.push(equation);
        equation.to_ptr(index)
    }

    pub fn iter(&self) -> ArenaIter<Equation,EquationPtr> {
        ArenaIter::new(&self.0)
    }
}
