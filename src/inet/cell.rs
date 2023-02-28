use core::panic;
use std::fmt::{Binary, Debug, Display, Formatter};

use super::{
    arena::{ArenaIter, ToPtr},
    symbol::{SymbolArity, SymbolBook, SymbolPtr},
    var::{BVars, FVars, VarItem, VarPtr, FreeStore, BoundStore, VarStore},
    BitSet32, BitSet64, Polarity,
};

#[derive(Debug, PartialEq)]
pub enum PortKind {
    Cell = 0,
    Var = 1,
}

impl From<u64> for PortKind {
    fn from(value: u64) -> Self {
        match value {
            0 => PortKind::Cell,
            1 => PortKind::Var,
            _ => panic!(),
        }
    }
}

impl From<u32> for PortKind {
    fn from(value: u32) -> Self {
        match value {
            0 => PortKind::Cell,
            1 => PortKind::Var,
            _ => panic!(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct PortPtr(u32);
impl PortPtr {
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
        this.set_kind(PortKind::Var);
        this.set_term(var_ptr.get_ptr());
        this
    }

    pub fn new_cell(cell_ptr: CellPtr) -> Self {
        let mut this = Self(0);
        this.set_kind(PortKind::Cell);
        this.set_term(cell_ptr.get_ptr());
        this
    }

    fn set_kind(&mut self, kind: PortKind) {
        self.0 = Self::KIND.set(self.0, kind as u32);
    }

    pub fn get_kind(&self) -> PortKind {
        PortKind::from(Self::KIND.get(self.0))
    }

    pub fn get_var(&self) -> VarPtr {
        assert!(self.get_kind() == PortKind::Var);
        VarPtr::from(self.get_term())
    }

    pub fn get_cell(&self) -> CellPtr {
        assert!(self.get_kind() == PortKind::Cell);
        CellPtr(self.get_term())
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

pub struct PortItem<'a, F: VarStore = FreeStore, B: VarStore = BoundStore> {
    pub port_ptr: PortPtr,
    pub symbols: &'a SymbolBook,
    pub cells: &'a Cells,
    pub bvars: &'a BVars<B>,
    pub fvars: &'a FVars<F>,
}
impl<'a,F: VarStore, B: VarStore> PortItem<'a,F,B> {
    fn to_cell_item(&self, cell_ptr: CellPtr) -> CellItem<'a, F, B> {
        CellItem {
            cell_ptr,
            symbols: self.symbols,
            cells: self.cells,
            bvars: self.bvars,
            fvars: self.fvars,
        }
    }
    fn to_var_item(&self, var_ptr: VarPtr) -> VarItem<'a, F, B> {
        VarItem { var_ptr, bvars: self.bvars, fvars: self.fvars }
    }
}
impl<'a, F: VarStore, B: VarStore> Display for PortItem<'a, F, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.port_ptr.get_kind() {
            PortKind::Cell => self.to_cell_item(self.port_ptr.get_cell()).fmt(f),
            PortKind::Var => self.to_var_item(self.port_ptr.get_var()).fmt(f),
        }
    }
}

impl Binary for PortPtr {
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

impl From<u64> for PortPtr {
    fn from(value: u64) -> Self {
        assert!(value <= 1 << 25);
        Self(value as u32)
    }
}

impl Debug for PortPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("PortPtr({:032b})", self.0);
        let mut b = f.debug_struct(&name);
        b.field("kind", &self.get_kind());
        match self.get_kind() {
            PortKind::Cell => b.field("cell", &self.get_cell()),
            PortKind::Var => b.field("var", &self.get_var()),
        };
        b.finish()
    }
}

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
        // new.set_kind(PortKind::Cell);
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
pub struct Cell(u64);
impl Cell {
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
        let mut cell = Self(0);
        cell.set_symbol_ptr(symbol_ptr);
        cell
    }

    pub fn new1(symbol_ptr: SymbolPtr, port: PortPtr) -> Self {
        let mut cell = Self(0);
        cell.set_symbol_ptr(symbol_ptr);
        cell.set_left_port(port);
        cell
    }

    pub fn new2(symbol_ptr: SymbolPtr, left_port: PortPtr, right_port: PortPtr) -> Self {
        let mut cell = Self(0);
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
    pub fn get_left_port(&self) -> PortPtr {
        PortPtr(Self::LEFT_PORT.get(self.0) as u32)
    }

    #[inline]
    fn set_left_port(&mut self, port: PortPtr) {
        self.0 = Self::LEFT_PORT.set(self.0, port.get_ptr() as u64)
    }

    #[inline]
    pub fn get_right_port(&self) -> PortPtr {
        PortPtr(Self::RIGHT_PORT.get(self.0) as u32)
    }

    fn set_right_port(&mut self, port: PortPtr) {
        self.0 = Self::RIGHT_PORT.set(self.0, port.get_ptr() as u64)
    }

    pub fn to_ptr(&self, index: usize) -> CellPtr {
        CellPtr::new(index, self.get_symbol_ptr().get_polarity())
    }
}

impl Binary for Cell {
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

impl ToPtr<CellPtr> for Cell {
    fn to_ptr(&self, index: usize) -> CellPtr {
        CellPtr::new(index, self.get_symbol_ptr().get_polarity())
    }
}

impl Debug for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("Cell({:064b})", self.0);
        let mut b = f.debug_struct(&name);
        b.field("symbol", &self.get_symbol_ptr());
        b.field("left", &self.get_left_port());
        b.field("right", &self.get_right_port());
        b.finish()
    }
}

pub struct CellItem<'a, F: VarStore = FreeStore, B: VarStore = BoundStore> {
    pub cell_ptr: CellPtr,
    pub symbols: &'a SymbolBook,
    pub cells: &'a Cells,
    pub bvars: &'a BVars<B>,
    pub fvars: &'a FVars<F>,
}
impl<'a, F: VarStore, B: VarStore> CellItem<'a, F, B> {
    fn to_port_item(&self, port_ptr: PortPtr) -> PortItem<'a, F, B> {
        PortItem {
            port_ptr: port_ptr,
            symbols: self.symbols,
            cells: self.cells,
            bvars: self.bvars,
            fvars: self.fvars,
        }
    }
}
impl<'a, F: VarStore, B: VarStore> Display for CellItem<'a, F, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cell = self.cells.get(self.cell_ptr);

        let name = self.symbols.get_name(cell.get_symbol_ptr());
        let symbol = self.symbols.get(cell.get_symbol_ptr());
        match symbol.get_arity() {
            SymbolArity::Zero => {
                write!(f, "{}", name)
            }
            SymbolArity::One => {
                write!(f, "({} {})", name, self.to_port_item(cell.get_left_port()))
            }
            SymbolArity::Two => {
                write!(
                    f,
                    "({} {} {})",
                    name,
                    self.to_port_item(cell.get_left_port()),
                    self.to_port_item(cell.get_right_port())
                )
            }
        }
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

    pub fn iter(&self) -> ArenaIter<Cell, CellPtr> {
        ArenaIter::new(&self.0)
    }

    pub fn get(&self, cell: CellPtr) -> &Cell {
        &self.0[cell.get_index()]
    }

    pub fn cell0(&mut self, symbol: SymbolPtr) -> CellPtr {
        self.add(Cell::new0(symbol))
    }

    pub fn reuse_cell0(&mut self, ptr: CellPtr, symbol: SymbolPtr) {
        self.0[ptr.get_index()] = Cell::new0(symbol);
    }

    pub fn cell1(&mut self, symbol: SymbolPtr, port: PortPtr) -> CellPtr {
        self.add(Cell::new1(symbol, port))
    }

    pub fn reuse_cell1(&mut self, ptr: CellPtr, symbol: SymbolPtr, port: PortPtr) {
        self.0[ptr.get_index()] = Cell::new1(symbol, port);
    }

    pub fn cell2(&mut self, symbol: SymbolPtr, left_port: PortPtr, right_port: PortPtr) -> CellPtr {
        self.add(Cell::new2(symbol, left_port, right_port))
    }

    pub fn reuse_cell2(
        &mut self,
        ptr: CellPtr,
        symbol: SymbolPtr,
        left_port: PortPtr,
        right_port: PortPtr,
    ) {
        self.0[ptr.get_index()] = Cell::new2(symbol, left_port, right_port);
    }

    pub fn add_all(&mut self, cells: Cells) {
        self.0.extend(cells.0)
    }

    fn add(&mut self, cell: Cell) -> CellPtr {
        let index = self.0.len();
        self.0.push(cell);
        cell.to_ptr(index)
    }
}

#[cfg(test)]
mod tests {
    use crate::inet::{symbol::SymbolArity, var::{FVarPtr, BVarPtr}};

    use super::*;

    #[test]
    fn test_cell0_neg() {
        let symbol_pos = SymbolPtr::new(2, SymbolArity::One, Polarity::Pos);
        let mut cell = Cell::new0(symbol_pos);
        assert_eq!(cell.get_symbol_ptr(), symbol_pos);
        let symbol_neg = SymbolPtr::new(2, SymbolArity::One, Polarity::Pos);
        cell.set_symbol_ptr(symbol_neg);
        assert_eq!(cell.get_symbol_ptr(), symbol_neg);
    }

    #[test]
    fn port_kind_from_u64() {
        assert_eq!(PortKind::Cell, PortKind::from(0u64));
        assert_eq!(PortKind::Var, PortKind::from(1u64));
        assert!(matches!(PortKind::from(0u64), PortKind::Cell));
        assert!(matches!(PortKind::from(1u64), PortKind::Var));
    }

    #[test]
    fn port_kind_from_u32() {
        assert_eq!(PortKind::Cell, PortKind::from(0u32));
        assert_eq!(PortKind::Var, PortKind::from(1u32));
        assert!(matches!(PortKind::from(0u32), PortKind::Cell));
        assert!(matches!(PortKind::from(1u32), PortKind::Var));
    }

    #[test]
    fn port_ptr_get_kind() {
        let fvar = FVarPtr::new(42);
        println!("{}", fvar.get_index());
        // println!("{}", fvar.get_ptr());
        println!("{:0b}", 8388650);
        println!("{:0b}", 25165866);

        let bvar = BVarPtr::new(42);
        let p1 = PortPtr::new_var(fvar.into());
        // println!("{}", p1.get_kind());
        // println!("{}", fvar.get_ptr());
        let p2 = PortPtr::new_var(bvar.into());
        let p3 = PortPtr::new_cell(CellPtr::new(42, Polarity::Pos));

        assert_eq!(PortKind::Var, p1.get_kind());
        assert_eq!(fvar, p1.into());
        assert_eq!(PortKind::Var, p2.get_kind());
        assert_eq!(bvar, p2.into());
        assert_eq!(PortKind::Cell, p3.get_kind());
    }

    #[test]
    fn port_ptr_get_fvar() {
        let fvar = FVarPtr::new(42);
        let p = PortPtr::new_var(fvar.into());

        assert_eq!(fvar, p.get_var().into());
    }

    #[test]
    fn port_ptr_get_bvar() {
        let bvar = BVarPtr::new(42);
        let p = PortPtr::new_var(bvar.into());

        assert_eq!(bvar, p.get_var().into());
    }

    #[test]
    fn port_ptr_get_cell() {
        let cell = CellPtr::new(42, Polarity::Pos);
        let p = PortPtr::new_cell(cell);

        assert_eq!(cell, p.get_cell());
    }

    #[test]
    #[should_panic]
    fn port_ptr_get_fvar_wrong_kind() {
        let cell = CellPtr::new(42, Polarity::Pos);
        let p = PortPtr::new_cell(cell);
        p.get_var();
    }

    #[test]
    #[should_panic]
    fn port_ptr_get_cell_wrong_kind() {
        let fvar = FVarPtr::new(42);
        let p = PortPtr::new_var(fvar.into());
        p.get_cell();
    }

    #[test]
    fn cell_ptr_get_ptr() {
        let cell_ptr = CellPtr::new(42, Polarity::Pos);
        assert_eq!(42, cell_ptr.get_ptr());
    }

    #[test]
    fn cell_ptr_get_polarity() {
        let cell_ptr1 = CellPtr::new(42, Polarity::Pos);
        assert_eq!(Polarity::Pos, cell_ptr1.get_polarity());

        let cell_ptr2 = CellPtr::new(42, Polarity::Neg);
        assert_eq!(Polarity::Neg, cell_ptr2.get_polarity());
    }

    #[test]
    fn cell2() {
        let sym_ptr = SymbolPtr::new(3, SymbolArity::One, Polarity::Neg);
        let r = FVarPtr::new(5);
        let a: PortPtr = r.into();
        println!("{:0b}", a);
        assert_eq!(PortKind::Var, a.get_kind());
        assert_eq!(a.get_var(), r.into());

        let cell = Cell::new1(sym_ptr, r.into());
        println!("{:0b}", cell);
        assert_eq!(PortKind::Var, cell.get_left_port().get_kind());
    }
}
