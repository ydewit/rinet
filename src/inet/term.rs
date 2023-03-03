use std::fmt::{Binary, Debug, Formatter};

use super::{
    cell::CellPtr,
    heap::Heap,
    symbol::SymbolBook,
    var::{Var, VarPtr},
    BitSet32,
};

pub trait TermFamily: Clone {
    type BoundStore: Default + Debug;
    type FreeStore: Default + Debug;

    fn display_store(
        f: &mut std::fmt::Formatter<'_>,
        symbols: &SymbolBook,
        heap: &Heap<Self>,
        store: &Var<Self>,
        index: usize,
    ) -> std::fmt::Result;
}

#[derive(Debug, PartialEq)]
pub enum TermKind {
    Cell = 0,
    Var = 1,
}

impl From<u64> for TermKind {
    fn from(value: u64) -> Self {
        match value {
            0 => TermKind::Cell,
            1 => TermKind::Var,
            _ => panic!(),
        }
    }
}

impl From<u32> for TermKind {
    fn from(value: u32) -> Self {
        match value {
            0 => TermKind::Cell,
            1 => TermKind::Var,
            _ => panic!(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TermPtr(u32);
impl TermPtr {
    // term can be a Var or a Cell
    const TERM: BitSet32<24> = BitSet32 {
        mask: 0b00000000_11111111_11111111_11111111,
        offset: 0,
    };
    const KIND: BitSet32<1> = BitSet32 {
        mask: 0b00000001,
        offset: 24,
    };
    const _UNUSED: BitSet32<7> = BitSet32 {
        mask: 0b11111110,
        offset: 25,
    };

    const PTR: BitSet32<25> = BitSet32 {
        mask: 0b00000001_11111111_11111111_11111111,
        offset: 0,
    };

    pub fn new_var(var_ptr: VarPtr) -> Self {
        let mut this = Self(0);
        this.set_kind(TermKind::Var);
        this.set_term(var_ptr.get_ptr());
        this
    }

    pub fn new_cell(cell_ptr: CellPtr) -> Self {
        let mut this = Self(0);
        this.set_kind(TermKind::Cell);
        this.set_term(cell_ptr.get_ptr());
        this
    }

    fn set_kind(&mut self, kind: TermKind) {
        self.0 = Self::KIND.set(self.0, kind as u32);
    }

    pub fn get_kind(&self) -> TermKind {
        TermKind::from(Self::KIND.get(self.0))
    }

    pub fn get_var_ptr(&self) -> VarPtr {
        assert!(self.get_kind() == TermKind::Var);
        self.get_term().into()
    }

    pub fn get_cell_ptr(&self) -> CellPtr {
        assert!(self.get_kind() == TermKind::Cell);
        self.get_term().into()
    }

    pub fn get_term(&self) -> u32 {
        Self::TERM.get(self.0)
    }

    fn set_term(&mut self, raw_ptr: u32) {
        self.0 = Self::TERM.set(self.0, raw_ptr)
    }

    pub fn get_ptr(&self) -> u32 {
        Self::PTR.get(self.0)
    }
}

impl Binary for TermPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:07b}_{:01b}_{:024b}",
            Self::_UNUSED.get(self.0),
            self.get_kind() as u8,
            self.get_term()
        )
    }
}

impl From<u64> for TermPtr {
    fn from(value: u64) -> Self {
        assert!(value <= 1 << 25);
        Self(value as u32)
    }
}

impl From<u32> for TermPtr {
    fn from(value: u32) -> Self {
        assert!(value <= 1 << 25);
        Self(value)
    }
}

impl Debug for TermPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("TermPtr({:032b})", self.0);
        let mut b = f.debug_struct(&name);
        b.field("kind", &self.get_kind());
        match self.get_kind() {
            TermKind::Cell => b.field("cell", &self.get_cell_ptr()),
            TermKind::Var => b.field("var", &self.get_var_ptr()),
        };
        b.finish()
    }
}
