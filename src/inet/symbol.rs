use std::{
    collections::HashMap,
    fmt::{Binary, Debug, Display, Formatter},
};

use super::{BitSet16, BitSet8, Polarity};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct SymbolName(pub &'static str);
// impl SymbolName {
//     pub fn new(name: &str) -> Self {
//         Self(name)
//     }
// }

impl Display for SymbolName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
impl From<&'static str> for SymbolName {
    fn from(value: &'static str) -> Self {
        Self(value)
    }
}
// impl From<String> for SymbolName {
//     fn from(value: String) -> Self {
//         Self(value)
//     }
// }

#[derive(Debug, PartialEq)]
pub enum SymbolArity {
    Zero = 0,
    One = 1,
    Two = 2,
}

impl From<u64> for SymbolArity {
    fn from(value: u64) -> Self {
        match value {
            0 => SymbolArity::Zero,
            1 => SymbolArity::One,
            2 => SymbolArity::Two,
            _ => panic!(),
        }
    }
}

impl From<u16> for SymbolArity {
    fn from(value: u16) -> Self {
        match value {
            0 => SymbolArity::Zero,
            1 => SymbolArity::One,
            2 => SymbolArity::Two,
            _ => panic!(),
        }
    }
}

impl From<u8> for SymbolArity {
    fn from(value: u8) -> Self {
        match value {
            0 => SymbolArity::Zero,
            1 => SymbolArity::One,
            2 => SymbolArity::Two,
            _ => panic!(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct SymbolPtr(u16);
impl SymbolPtr {
    const INDEX: BitSet16<11> = BitSet16 {
        mask: 0b00000111_11111111,
        offset: 0,
    };
    const POLARITY: BitSet16<1> = BitSet16 {
        mask: 0b00001,
        offset: 11,
    };
    const ARITY: BitSet16<2> = BitSet16 {
        mask: 0b0011,
        offset: 12,
    };

    const PTR: BitSet16<14> = BitSet16 {
        mask: 0b00111111_11111111,
        offset: 0,
    };

    pub fn new(index: usize, arity: SymbolArity, polarity: Polarity) -> Self {
        let mut new = Self(0);
        new.set_index(index);
        new.set_polarity(polarity);
        new.set_arity(arity);
        new
    }

    #[inline]
    pub fn get_polarity(self) -> Polarity {
        Polarity::from(Self::POLARITY.get(self.0))
    }

    #[inline]
    fn set_polarity(&mut self, polarity: Polarity) {
        self.0 = Self::POLARITY.set(self.0, polarity as u16)
    }

    #[inline]
    pub fn get_index(&self) -> usize {
        Self::INDEX.get(self.0) as usize
    }

    #[inline]
    fn set_index(&mut self, index: usize) {
        self.0 = Self::INDEX.set(self.0, index as u16)
    }

    pub fn get_arity(&self) -> SymbolArity {
        SymbolArity::from(Self::ARITY.get(self.0))
    }

    fn set_arity(&mut self, arity: SymbolArity) {
        self.0 |= Self::ARITY.set(self.0, arity as u16);
    }

    #[inline]
    pub fn get_ptr(&self) -> u16 {
        Self::PTR.get(self.0)
    }

    pub fn get_raw(&self) -> u16 {
        self.0
    }
}

impl Binary for SymbolPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02b}_{:01b}_{:013b}",
            self.get_arity() as u8,
            self.get_polarity() as u8,
            self.get_index()
        )
    }
}

impl From<u64> for SymbolPtr {
    fn from(value: u64) -> Self {
        SymbolPtr(value as u16)
    }
}

impl Into<u64> for SymbolPtr {
    fn into(self) -> u64 {
        self.get_ptr() as u64
    }
}

impl Debug for SymbolPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("SymbolPtr({:016b})", self.0);
        let mut b = f.debug_struct(&name);
        b.field("polarity", &self.get_polarity());
        b.field("index", &self.get_index());
        b.finish()
    }
}

#[derive(Clone, Copy)]
pub struct Symbol(u8);
impl Symbol {
    //                                              0bAAPLR???
    const ARITY: BitSet8<2> = BitSet8 {
        mask: 0b11,
        offset: 6,
    };
    const POLARITY: BitSet8<1> = BitSet8 {
        mask: 0b001,
        offset: 5,
    };
    const LEFT_POLARITY: BitSet8<1> = BitSet8 {
        mask: 0b0001,
        offset: 4,
    };
    const RIGHT_POLARITY: BitSet8<1> = BitSet8 {
        mask: 0b00001,
        offset: 3,
    };

    pub fn new0(polarity: Polarity) -> Self {
        let mut sym = Self(0);
        sym.set_arity(SymbolArity::Zero);
        sym.set_polarity(polarity);
        sym
    }

    pub fn new1(polarity: Polarity, port_polarity: Polarity) -> Self {
        let mut sym = Self(0);
        sym.set_arity(SymbolArity::One);
        sym.set_polarity(polarity);
        sym.set_left_polarity(port_polarity);
        sym
    }

    pub fn new2(polarity: Polarity, left_polarity: Polarity, right_polarity: Polarity) -> Self {
        let mut sym = Self(0);
        sym.set_arity(SymbolArity::Two);
        sym.set_polarity(polarity);
        sym.set_left_polarity(left_polarity);
        sym.set_right_polarity(right_polarity);
        sym
    }

    #[inline]
    pub fn get_arity(&self) -> SymbolArity {
        SymbolArity::from(Self::ARITY.get(self.0))
    }

    #[inline]
    fn set_arity(&mut self, arity: SymbolArity) {
        self.0 = Self::ARITY.set(self.0, arity as u8)
    }

    #[inline]
    pub fn get_polarity(&self) -> Polarity {
        Polarity::from(Self::POLARITY.get(self.0))
    }

    #[inline]
    fn set_polarity(&mut self, polarity: Polarity) {
        self.0 = Self::POLARITY.set(self.0, polarity as u8)
    }

    #[inline]
    pub fn get_left_polarity(&self) -> Polarity {
        Polarity::from(Self::LEFT_POLARITY.get(self.0))
    }

    #[inline]
    fn set_left_polarity(&mut self, polarity: Polarity) {
        assert!(self.get_arity() == SymbolArity::One || self.get_arity() == SymbolArity::Two);
        self.0 = Self::LEFT_POLARITY.set(self.0, polarity as u8)
    }

    #[inline]
    pub fn get_right_polarity(&self) -> Polarity {
        assert!(self.get_arity() == SymbolArity::Two);
        Polarity::from(Self::RIGHT_POLARITY.get(self.0))
    }

    #[inline]
    fn set_right_polarity(&mut self, polarity: Polarity) {
        assert!(self.get_arity() == SymbolArity::Two);
        self.0 = Self::RIGHT_POLARITY.set(self.0, polarity as u8)
    }

    #[inline]
    pub fn to_ptr(&self, index: usize) -> SymbolPtr {
        SymbolPtr::new(index, self.get_arity(), self.get_polarity())
    }
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("Symbol({:08b})", self.0);
        let mut b = f.debug_struct(&name);
        b.field("arity", &self.get_arity());
        b.field("polarity", &self.get_polarity());
        match self.get_arity() {
            SymbolArity::Zero => {}
            SymbolArity::One => {
                b.field("port", &self.get_left_polarity());
            }
            SymbolArity::Two => {
                b.field("left", &self.get_left_polarity());
                b.field("right", &self.get_right_polarity());
            }
        };
        b.finish()
    }
}

pub struct SymbolItem<'a> {
    pub symbol_ptr: SymbolPtr,
    pub symbols: &'a SymbolBook,
}
impl<'a> Display for SymbolItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.symbols.get_name(self.symbol_ptr).unwrap();
        let symbol = self.symbols.get(self.symbol_ptr);
        match symbol.get_arity() {
            SymbolArity::Zero => {
                write!(f, "{}", name)
            }
            SymbolArity::One => {
                write!(f, "({} {})", name, symbol.get_left_polarity())
            }
            SymbolArity::Two => {
                write!(
                    f,
                    "({} {} {})",
                    name,
                    symbol.get_left_polarity(),
                    symbol.get_right_polarity()
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct SymbolBook {
    symbols: Vec<Symbol>,
    symbol_by_name: HashMap<SymbolName, usize>,
    name_by_symbol: HashMap<usize, SymbolName>,
}

impl SymbolBook {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            symbol_by_name: HashMap::new(),
            name_by_symbol: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn ctr0(&mut self, name: &SymbolName) -> SymbolPtr {
        self.declare0(name, Polarity::Pos)
    }

    pub fn ctr1(&mut self, name: &SymbolName, port_polarity: Polarity) -> SymbolPtr {
        self.declare1(name, Polarity::Pos, port_polarity)
    }

    pub fn ctr2(&mut self, name: &SymbolName, left_polarity: Polarity, right_polarity: Polarity) -> SymbolPtr {
        self.declare2(name, Polarity::Pos, left_polarity, right_polarity)
    }

    pub fn fun0(&mut self, name: &SymbolName) -> SymbolPtr {
        self.declare0(name, Polarity::Neg)
    }

    pub fn fun1(&mut self, name: &SymbolName, port_polarity: Polarity) -> SymbolPtr {
        self.declare1(name, Polarity::Neg, port_polarity)
    }

    pub fn fun2(&mut self, name: &SymbolName, left_polarity: Polarity, right_polarity: Polarity) -> SymbolPtr {
        self.declare2(name, Polarity::Neg, left_polarity, right_polarity)
    }

    fn declare0(&mut self, name: &SymbolName, polarity: Polarity) -> SymbolPtr {
        let ptr = self.add_symbol(Symbol::new0(polarity));
        self.symbol_by_name
            .insert(name.clone(), ptr.get_index());
        self.name_by_symbol.insert(ptr.get_index(), name.clone());
        ptr
    }

    fn declare1(
        &mut self,
        name: &SymbolName,
        polarity: Polarity,
        left_port_polarity: Polarity,
    ) -> SymbolPtr {
        let ptr = self.add_symbol(Symbol::new1(polarity, left_port_polarity));
        self.symbol_by_name
            .insert(name.clone(), ptr.get_index());
        self.name_by_symbol.insert(ptr.get_index(), name.clone());
        ptr
    }

    fn declare2(
        &mut self,
        name: &SymbolName,
        polarity: Polarity,
        left_port_polarity: Polarity,
        right_port_polarity: Polarity,
    ) -> SymbolPtr {
        let ptr = self.add_symbol(Symbol::new2(
            polarity,
            left_port_polarity,
            right_port_polarity,
        ));
        self.symbol_by_name
            .insert(name.clone(), ptr.get_index());
        self.name_by_symbol.insert(ptr.get_index(), name.clone());
        ptr
    }

    pub fn get(&self, symbol_ptr: SymbolPtr) -> Symbol {
        self.symbols[symbol_ptr.get_index()]
    }

    pub fn get_by_name(&self, name: &SymbolName) -> Option<SymbolPtr> {
        match self.symbol_by_name.get(&name) {
            Some(index) => {
                let symbol = self.symbols[*index];
                Some(symbol.to_ptr(*index))
            }
            None => None,
        }
    }

    fn add_symbol(&mut self, symbol: Symbol) -> SymbolPtr {
        let index = self.symbols.len();
        let ptr = symbol.to_ptr(index);
        self.symbols.push(symbol);
        ptr
    }

    pub fn get_name(&self, symbol: SymbolPtr) -> Option<SymbolName> {
        self.name_by_symbol.get(&symbol.get_index()).cloned()
    }

    pub fn iter(&self) -> impl Iterator<Item = NamedSymbol> {
        self.symbols.iter().enumerate().map(|(index, symbol)| {
            let name = self.name_by_symbol.get(&index).cloned().unwrap();
            NamedSymbol {
                index,
                symbol,
                name,
            }
        })
    }

    pub fn display_symbol<'a>(&'a self, symbol_ptr: SymbolPtr) -> SymbolItem {
        SymbolItem {
            symbol_ptr,
            symbols: self,
        }
    }
}

pub struct NamedSymbol<'a> {
    index: usize,
    name: SymbolName,
    symbol: &'a Symbol,
}

impl<'a> Display for NamedSymbol<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.symbol.get_arity() {
            SymbolArity::Zero => write!(
                f,
                "Symbol[{}]: {}{}",
                self.index,
                self.symbol.get_polarity(),
                self.name
            ),
            SymbolArity::One => write!(
                f,
                "Symbol[{}]: {}({} {})",
                self.index,
                self.symbol.get_polarity(),
                self.name,
                self.symbol.get_left_polarity()
            ),
            SymbolArity::Two => write!(
                f,
                "Symbol[{}]: {}({} {} {})",
                self.index,
                self.symbol.get_polarity(),
                self.name,
                self.symbol.get_left_polarity(),
                self.symbol.get_right_polarity()
            ),
        }
    }
}

impl Display for SymbolBook {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for named_symbol in self.iter() {
            match writeln!(f, "{}", named_symbol) {
                Ok(_) => (),
                Err(_) => panic!(),
            }
        }
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_ptr_new0_neg() {
        let ptr = SymbolPtr::new(1, SymbolArity::One, Polarity::Neg);
        assert_eq!(ptr.get_index(), 1);
        assert_eq!(ptr.get_polarity(), Polarity::Neg);
        assert_eq!(ptr.get_arity(), SymbolArity::One);
    }

    #[test]
    fn test_symbol_ptr_new0_pos() {
        let ptr = SymbolPtr::new(1, SymbolArity::Two, Polarity::Pos);
        assert_eq!(ptr.get_index(), 1);
        assert_eq!(ptr.get_polarity(), Polarity::Pos);
        assert_eq!(ptr.get_arity(), SymbolArity::Two);
    }

    #[test]
    fn test_symbol_new0_neg() {
        let sym = Symbol::new0(Polarity::Neg);
        println!("{:0b}", sym.0);
        assert_eq!(sym.get_arity(), SymbolArity::Zero);
        assert_eq!(sym.get_polarity(), Polarity::Neg);
    }

    #[test]
    fn test_symbol_new0_pos() {
        let sym = Symbol::new0(Polarity::Pos);
        assert_eq!(sym.get_arity(), SymbolArity::Zero);
        assert_eq!(sym.get_polarity(), Polarity::Pos);
    }

    #[test]
    fn test_symbol_new1_neg_pos() {
        let sym = Symbol::new1(Polarity::Neg, Polarity::Pos);
        assert_eq!(sym.get_arity(), SymbolArity::One);
        assert_eq!(sym.get_polarity(), Polarity::Neg);
        assert_eq!(sym.get_left_polarity(), Polarity::Pos);
    }

    #[test]
    fn test_symbol_new1_neg_neg() {
        let sym = Symbol::new1(Polarity::Neg, Polarity::Neg);
        assert_eq!(sym.get_arity(), SymbolArity::One);
        assert_eq!(sym.get_polarity(), Polarity::Neg);
        assert_eq!(sym.get_left_polarity(), Polarity::Neg);
    }

    #[test]
    fn test_symbol_new1_pos_pos() {
        let sym = Symbol::new1(Polarity::Pos, Polarity::Pos);
        assert_eq!(sym.get_arity(), SymbolArity::One);
        assert_eq!(sym.get_polarity(), Polarity::Pos);
        assert_eq!(sym.get_left_polarity(), Polarity::Pos);
    }

    #[test]
    fn test_symbol_new1_pos_neg() {
        let sym = Symbol::new1(Polarity::Pos, Polarity::Neg);
        assert_eq!(sym.get_arity(), SymbolArity::One);
        assert_eq!(sym.get_polarity(), Polarity::Pos);
        assert_eq!(sym.get_left_polarity(), Polarity::Neg);
    }

    #[test]
    fn test_symbol_new2_pos_pos_pos() {
        let sym = Symbol::new2(Polarity::Pos, Polarity::Pos, Polarity::Pos);
        assert_eq!(sym.get_arity(), SymbolArity::Two);
        assert_eq!(sym.get_polarity(), Polarity::Pos);
        assert_eq!(sym.get_left_polarity(), Polarity::Pos);
        assert_eq!(sym.get_right_polarity(), Polarity::Pos);
    }

    #[test]
    fn test_symbol_new2_pos_pos_neg() {
        let sym = Symbol::new2(Polarity::Pos, Polarity::Pos, Polarity::Neg);
        assert_eq!(sym.get_arity(), SymbolArity::Two);
        assert_eq!(sym.get_polarity(), Polarity::Pos);
        assert_eq!(sym.get_left_polarity(), Polarity::Pos);
        assert_eq!(sym.get_right_polarity(), Polarity::Neg);
    }

    #[test]
    fn test_symbol_new2_pos_neg_neg() {
        let sym = Symbol::new2(Polarity::Pos, Polarity::Neg, Polarity::Neg);
        assert_eq!(sym.get_arity(), SymbolArity::Two);
        assert_eq!(sym.get_polarity(), Polarity::Pos);
        assert_eq!(sym.get_left_polarity(), Polarity::Neg);
        assert_eq!(sym.get_right_polarity(), Polarity::Neg);
    }

    #[test]
    fn test_symbol_new2_neg_neg_neg() {
        let sym = Symbol::new2(Polarity::Neg, Polarity::Neg, Polarity::Neg);
        assert_eq!(sym.get_arity(), SymbolArity::Two);
        assert_eq!(sym.get_polarity(), Polarity::Neg);
        assert_eq!(sym.get_left_polarity(), Polarity::Neg);
        assert_eq!(sym.get_right_polarity(), Polarity::Neg);
    }

    #[test]
    fn test_symbol_new2_neg_pos_neg() {
        let sym = Symbol::new2(Polarity::Neg, Polarity::Pos, Polarity::Neg);
        assert_eq!(sym.get_arity(), SymbolArity::Two);
        assert_eq!(sym.get_polarity(), Polarity::Neg);
        assert_eq!(sym.get_left_polarity(), Polarity::Pos);
        assert_eq!(sym.get_right_polarity(), Polarity::Neg);
    }

    #[test]
    fn test_symbol_new2_neg_pos_pos() {
        let sym = Symbol::new2(Polarity::Neg, Polarity::Pos, Polarity::Pos);
        assert_eq!(sym.get_arity(), SymbolArity::Two);
        assert_eq!(sym.get_polarity(), Polarity::Neg);
        assert_eq!(sym.get_left_polarity(), Polarity::Pos);
        assert_eq!(sym.get_right_polarity(), Polarity::Pos);
    }

    #[test]
    fn test_symbol_new2_neg_neg_pos() {
        let sym = Symbol::new2(Polarity::Neg, Polarity::Neg, Polarity::Pos);
        assert_eq!(sym.get_arity(), SymbolArity::Two);
        assert_eq!(sym.get_polarity(), Polarity::Neg);
        assert_eq!(sym.get_left_polarity(), Polarity::Neg);
        assert_eq!(sym.get_right_polarity(), Polarity::Pos);
    }

    #[test]
    #[should_panic]
    fn test_symbol_from_left_panic() {
        let symbol = Symbol::new0(Polarity::Neg);
        symbol.get_left_polarity();
    }

    #[test]
    #[should_panic]
    fn test_symbol_from_right_panic() {
        let symbol = Symbol::new0(Polarity::Neg);
        symbol.get_right_polarity();
    }

    #[test]
    #[should_panic]
    fn test_symbol1_from_left_panic() {
        let symbol = Symbol::new1(Polarity::Neg, Polarity::Neg);
        symbol.get_right_polarity();
    }

    #[test]
    fn test_symbol_ptr_new_set_index() {
        let mut ptr = SymbolPtr::new(0, SymbolArity::Zero, Polarity::Pos);
        ptr.set_index(100);
        assert_eq!(ptr.get_index(), 100);
    }

    #[test]
    fn test_symbol_ptr_new_set_polarity() {
        let mut ptr = SymbolPtr::new(0, SymbolArity::One, Polarity::Pos);
        ptr.set_polarity(Polarity::Neg);
        assert_eq!(ptr.get_polarity(), Polarity::Neg);
    }
}
