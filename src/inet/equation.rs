use std::{
    fmt::{Binary, Debug, Display, Formatter},
    marker::PhantomData,
};

use crate::inet::Polarity;

use super::{
    arena::{Arena, ArenaPtr, ArenaValue},
    cell::CellPtr,
    heap::Heap,
    net::NetF,
    symbol::{SymbolBook, SymbolName},
    term::{TermFamily, TermPtr},
    var::PVarPtr,
    BitSet32, BitSet64,
};

#[derive(Debug, PartialEq)]
pub enum EquationKind {
    Redex = 0,
    Bind = 1,
    Connect = 2,
}
impl Display for EquationKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EquationKind::Redex => f.write_str("REDEX"),
            EquationKind::Bind => f.write_str("BIND"),
            EquationKind::Connect => f.write_str("CONNECT"),
        }
    }
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

impl From<u32> for EquationKind {
    fn from(value: u32) -> Self {
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
pub struct EquationPtr(u32);
impl EquationPtr {
    const INDEX: BitSet32<30> = BitSet32 {
        mask: 0b00111111_11111111_11111111_11111111,
        offset: 0,
    };
    const KIND: BitSet32<2> = BitSet32 {
        mask: 0b11,
        offset: 30,
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
        self.0 |= Self::KIND.set(self.0, kind as u32)
    }

    #[inline]
    pub fn get_index(&self) -> usize {
        Self::INDEX.get(self.0) as usize
    }

    fn set_index(&mut self, index: usize) {
        assert!(index < (u32::MAX - Self::KIND.mask as u32) as usize);
        self.0 |= Self::INDEX.set(self.0, index as u32)
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
pub struct Equation<T: TermFamily>(pub u64, pub PhantomData<T>);
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
        eqn.set_kind(EquationKind::Redex);
        eqn.set_left(left.get_ptr());
        eqn.set_right(right.get_ptr());
        eqn
    }

    pub fn bind(var: PVarPtr, cell: CellPtr) -> Self {
        let mut eqn = Equation(0, PhantomData);
        eqn.set_kind(EquationKind::Bind);
        eqn.set_left(var.get_ptr());
        eqn.set_right(cell.get_ptr());
        eqn
    }

    pub fn connect(left: PVarPtr, right: PVarPtr) -> Self {
        let mut eqn = Equation(0, PhantomData);
        eqn.set_kind(EquationKind::Connect);
        eqn.set_left(left.get_ptr());
        eqn.set_right(right.get_ptr());
        eqn
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
    pub fn get_bind_var(&self) -> PVarPtr {
        assert!(self.get_kind() == EquationKind::Bind);
        PVarPtr::from(self.get_left())
    }

    #[inline]
    pub fn get_bind_cell(&self) -> CellPtr {
        assert!(self.get_kind() == EquationKind::Bind);
        CellPtr::from(self.get_right())
    }

    #[inline]
    pub fn get_connect_left(&self) -> PVarPtr {
        assert!(self.get_kind() == EquationKind::Connect);
        PVarPtr::from(self.get_left())
    }

    #[inline]
    pub fn get_connect_right(&self) -> PVarPtr {
        assert!(self.get_kind() == EquationKind::Connect);
        PVarPtr::from(self.get_right())
    }

    pub fn display_equation<'a>(&'a self, symbols: &'a SymbolBook, heap: &'a Heap<T>) -> EquationDisplay<T> {
        EquationDisplay {
            equation: self,
            symbols,
            heap,
        }
    }

}

impl<T: TermFamily> ArenaValue<EquationPtr> for Equation<T> {
    fn to_ptr(&self, index: usize) -> EquationPtr {
        EquationPtr::new(index, self.get_kind())
    }
}

impl<T: TermFamily> From<u64> for Equation<T> {
    fn from(value: u64) -> Self {
        Equation(value, PhantomData)
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
    pub equation: &'a Equation<T>,
    pub symbols: &'a SymbolBook,
    pub heap: &'a Heap<T>,
}
impl<'a, T: TermFamily> Display for EquationDisplay<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.equation.get_kind() {
            EquationKind::Redex => {
                write!(
                    f,
                    "{} = {}",
                    self.heap
                        .display_cell(self.symbols, self.equation.get_redex_fun()),
                    self.heap
                        .display_cell(self.symbols, self.equation.get_redex_ctr())
                )
            }
            EquationKind::Bind => {
                write!(
                    f,
                    "{} ← {}",
                    self.heap
                        .display_var(self.symbols, self.equation.get_bind_var().get_fvar_ptr()),
                    self.heap
                        .display_cell(self.symbols, self.equation.get_bind_cell())
                )
            }
            EquationKind::Connect => {
                write!(
                    f,
                    "{} ↔ {}",
                    self.heap
                        .display_var(self.symbols, self.equation.get_connect_left().get_fvar_ptr()),
                    self.heap
                        .display_var(self.symbols, self.equation.get_connect_right().get_fvar_ptr())
                )
            }
        }
    }
}

pub type Equations<T> = Arena<Equation<T>, EquationPtr>;

pub struct EquationsDisplay<'a, T: TermFamily> {
    pub symbols: &'a SymbolBook,
    pub body: &'a Vec<Equation<T>>,
    pub heap: &'a Heap<T>,
}
impl<'a, T: TermFamily> EquationsDisplay<'a, T> {
    fn to_equation_item(&'a self, equation: &'a Equation<T>) -> EquationDisplay<T> {
        EquationDisplay {
            equation,
            symbols: self.symbols,
            heap: self.heap,
        }
    }
}
impl<'a, T: TermFamily> Display for EquationsDisplay<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.body.iter().fold(Ok(()), |result, eqn| {
            result.and_then(|_| write!(f, " {}", self.to_equation_item(eqn)))
        })
    }
}

pub struct EquationBuilder<'a, F: TermFamily = NetF> {
    symbols: &'a SymbolBook,
    head: &'a mut Vec<PVarPtr>,
    equations: &'a mut Vec<Equation<F>>,
    heap: &'a mut Heap<F>,
}
impl<'a, F: TermFamily> EquationBuilder<'a, F> {
    pub(crate) fn new(
        symbols: &'a SymbolBook,
        head: &'a mut Vec<PVarPtr>,
        equations: &'a mut Vec<Equation<F>>,
        heap: &'a mut Heap<F>,
    ) -> Self {
        Self {
            symbols,
            head,
            equations,
            heap,
        }
    }

    pub fn redex(&mut self, ctr_ptr: CellPtr, fun_ptr: CellPtr) {
        assert!(ctr_ptr.get_polarity() == Polarity::Pos);
        assert!(fun_ptr.get_polarity() == Polarity::Neg);
        self.equations.push(Equation::redex(ctr_ptr, fun_ptr))
    }

    pub fn bind(&mut self, var_ptr: PVarPtr, cell_ptr: CellPtr) {
        self.equations.push(Equation::bind(var_ptr, cell_ptr))
    }

    pub fn connect(&mut self, left_ptr: PVarPtr, right_ptr: PVarPtr) {
        assert!(
            left_ptr.get_polarity() != right_ptr.get_polarity(),
            "Cannot connect vars with same polarity"
        );
        self.equations.push(Equation::connect(left_ptr, right_ptr))
    }

    // ----------------

    pub fn cell0(&mut self, name: &SymbolName) -> CellPtr {
        let symbol_ptr = self.symbols.get_by_name(name).unwrap(); // TODO better error handling
        self.heap.cell0(symbol_ptr)
    }

    pub fn cell1(&mut self, name: &SymbolName, left_port: TermPtr) -> CellPtr {
        let symbol_ptr = self.symbols.get_by_name(name).unwrap(); // TODO better error handling
        let symbol = self.symbols.get(symbol_ptr);
        // check left polarity
        assert!(left_port
            .get_polarity()
            .is_opposite(symbol.get_left_polarity()));
        self.heap.cell1(symbol_ptr, left_port)
    }

    pub fn cell2(&mut self, name: &SymbolName, left_port: TermPtr, right_port: TermPtr) -> CellPtr {
        let symbol_ptr = self.symbols.get_by_name(name).unwrap(); // TODO better error handling
        let symbol = self.symbols.get(symbol_ptr);
        // check left polarity
        assert!(left_port
            .get_polarity()
            .is_opposite(symbol.get_left_polarity()));
        // check right polarity
        assert!(right_port
            .get_polarity()
            .is_opposite(symbol.get_right_polarity()));
        self.heap.cell2(symbol_ptr, left_port, right_port)
    }

    // -------------------

    pub fn input(&mut self) -> PVarPtr {
        let fvar_ptr = self.heap.fvar(F::FreeStore::default());
        let (neg_pvar, pos_pvar) = PVarPtr::wire(fvar_ptr);
        self.head.push(neg_pvar);
        pos_pvar // input fvars need to be "consumed" by the net (input from an inside-pov)
    }

    pub fn output(&mut self) -> PVarPtr {
        let fvar_ptr = self.heap.fvar(F::FreeStore::default());
        let (neg_pvar, pos_pvar) = PVarPtr::wire(fvar_ptr);
        self.head.push(pos_pvar);
        neg_pvar // output fvars need to be "produced" by the net (output from an inside-pov)
    }

    pub fn var(&mut self) -> (PVarPtr, PVarPtr) {
        let bvar_ptr = self.heap.bvar(F::BoundStore::default());
        PVarPtr::wire(bvar_ptr)
    }

    // -------------------

    pub(crate) fn build(self) -> Self {
        self
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
