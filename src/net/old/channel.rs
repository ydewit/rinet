use std::{marker::PhantomData, ptr::NonNull};

use super::Polarity;

type Instr = u64;

struct BitField<T: TryFrom<Bits>> {
    offset: u64,
    mask: u64,
    phantom: PhantomData<T>
}

impl<T: TryFrom<Bits>> BitField<T> {
    fn new(offset: u64, mask: u64) -> Self{
        Self { offset, mask, phantom: PhantomData }
    }

    fn set(self, value: T) -> Bits {
        let bits = Bits::from(value) * self.offset;
        assert!(bits & self.mask == bits);
        bits
    }

    fn get(self, bits: Bits) -> Option<T> {
        From::<Bits<T>>::try_from((bits.0 / self.offset) & self.mask)
    }
}

#[derive(Debug)]
struct Bits(u64);

impl Bits {
    fn new(value: u64) -> Self {
        Self( value )
    }
}


impl TryFrom<Bits> for Polarity {
    type Error = Bits;
    fn try_from(value: Bits) -> Result<Self, Self::Error> {
        if value.0 == 1 {
            Ok(Polarity::Pos)
        }
        else if value.0 == 2 {
            Ok(Polarity::Neg)
        }
        else {
            Err(value)
        }
    }
}
impl From<Polarity> for Bits {
    fn from(value: Polarity) -> Self {
        match value {
            Polarity::Pos => Bits::new(1),
            Polarity::Neg => Bits::new(2),
        }
    }
}

#[derive(Debug)]
enum TermTag {
    Cell,
    Var
}
impl TryFrom<Bits> for TermTag {
    type Error = Bits;
    fn try_from(value: Bits) -> Result<Self, Self::Error> {
        if value.0 == 0 {
            Ok(TermTag::Cell)
        } else if value.0 == 1 {
            Ok(TermTag::Var)
        } else {
            Err(value)
        }
    }
}
impl From<TermTag> for Bits {
    fn from(value: TermTag) -> Self {
        match value {
            TermTag::Cell => Bits::new(0),
            TermTag::Var => Bits::new(1),
        }
    }
}

trait Term {
    const FIELD_TAG : BitField<TermTag> = BitField::new(0b10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000, 0b1);

    fn get_tag(&self) -> TermTag;
}


#[derive(Debug)]
struct TermId(u32);

impl TryFrom<Bits> for TermId {
    type Error = Bits;
    fn try_from(value: Bits) -> Result<Self, Self::Error> {
        Ok(TermId(value.0 as u32))
    }
}

impl From<TermId> for Bits {
    fn from(value: TermId) -> Self {
        Bits::new(value.0 as u64)
    }
}


#[derive(Debug)]
struct SymbolId(u16);

impl TryFrom<Bits> for SymbolId {
    type Error = Bits;
    fn try_from(value: Bits) -> Result<Self, Self::Error> {
        NonNull
        Ok(SymbolId(value.0 as u16))
    }
}

impl From<SymbolId> for Bits {
    fn from(value: SymbolId) -> Self {
        Bits::new(value.0 as u64)
    }
}


#[derive(Debug)]
struct Cell(Bits);

const NULL_TERM_ID: TermId = TermId(0);

impl Cell {
    const FIELD_SYMBOL_ID : BitField<SymbolId> = BitField::new(0b00000000_10000000_00000000_00000000_00000000_00000000_00000000_00000000, 0b01111111_1111);
    const FIELD_PORT_0    : BitField<TermId> = BitField::new(0b00000000_00000000_00000000_00000000_00000001_00000000_00000000_00000000, 0b00000000_00001111_11111111_11111111_1111);
    const FIELD_PORT_1    : BitField<TermId> = BitField::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001, 0b00000000_00000000_00000000_00000000_00001111_11111111_11111111_11111111);

    pub fn new(symbol_id: SymbolId, port_0: Option<TermId>, port_1: Option<TermId>) -> Self {
        let tag = Self::FIELD_TAG.set(TermTag::Cell);
        let symbol_id = Bits::from(symbol_id);
        Self::FIELD_SYMBOL_ID.set(symbol_id);
        let port_0 = port_0.map(Cell::FIELD_PORT_0::set).unwrap_or(0);
        let port_1 = port_1.map(Self::FIELD_PORT_1::set).unwrap_or(0);
        Self(tag | symbol_id | port_0 | port_1)
    }

    pub fn get_symbol_id(&self) -> SymbolId {
        Cell::FIELD_SYMBOL_ID.get(self.0).unwrap()
    }

    pub fn get_port_0(&self) -> Option<TermId> {
        Cell::FIELD_PORT_0.get(self.0)
    }

    pub fn get_port_1(&self) -> Option<TermId> {
        Cell::FIELD_PORT_1.get(self.0)
    }
}

impl Term for Cell {
    fn get_tag(&self) -> TermTag {
        TermTag::Cell
    }
}

#[derive(Debug)]
struct Var(u64);

// impl TryFrom<Bits<Var>> for Var {
//     type Error = Bits<TermId>;
//     fn try_from(value: Bits<Var>) -> Result<Self, Self::Error> {
//         Ok(Var(value.0))
//     }
// }

// impl From<Var> for Bits<Var> {
//     fn from(value: Var) -> Self {
//         Bits::new(value.0)
//     }
// }


#[derive(Debug)]
struct ValueId(u64);

impl TryFrom<Bits> for ValueId {
    type Error = Bits;
    fn try_from(value: Bits) -> Result<Self, Self::Error> {
        Ok(ValueId(value.0))
    }
}

impl From<ValueId> for Bits {
    fn from(value: ValueId) -> Self {
        Bits::new(value.0)
    }
}

impl Var {
    const FIELD_POLARITY : BitField<Polarity> = BitField::new(0b00100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000, 0b011);
    const FIELD_VALUE_ID : BitField<ValueId> = BitField::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001, 0b00000000_00000000_00000000_00000000_00000000_11111111_11111111_11111111);

    pub fn new(polarity: Option<Polarity>, value_id: ValueId) -> Self {
        let tag = Self::FIELD_TAG.set(TermTag::Var);
        let polarity = match polarity {
            Some(polarity) => Var::FIELD_POLARITY.set(polarity),
            None => todo!(),
        };
        Var::FIELD_POLARITY.set(polarity.map(Polarity::to_Instr).unwrap_or(0));
        let value_id = Var::FIELD_VALUE_ID.set(value_id.to_Instr());
        Self(Self::TAG | polarity | value_id)
    }

    pub fn get_polarity(&self) -> Option<Polarity> {
        let val = (self.to_Instr() / VAR_POLARITY) & VAR_POLARITY_MASK;
        Polarity::try_from_Instr(val)
    }

    pub fn get_value_id(&self) -> ValueId {
        ValueId((self.to_Instr() / VAR_VALUE_ID) & VAR_VALUE_ID_MASK)
    }
}

impl Term for Var {
    const TAG: u64 = 0x1;

    fn get_tag(&self) -> TermTag {
        TermTag::Var
    }

    fn to_Instr(&self) -> Instr {
        self.0
    }
}


#[test]
fn main() {
    // Bitwise operations
    println!("0011 AND 0101 is {:04b}", 0b0011u32 & 0b0101);
    println!("0011 OR 0101 is {:04b}", 0b0011u32 | 0b0101);
    println!("0011 XOR 0101 is {:04b}", 0b0011u32 ^ 0b0101);
    println!("1 << 5 is {}", 1u32 << 5);
    println!("0b{:0b} >> 2 is 0b{:0b}",8u32, 8u32 >> 2);
    println!("0b{:0b} << 2 is 0b{:0b}",8u32, 8u32 << 2);
    println!("{:0}", 0x1000000000000000u64 );
    // println!("{:?}", TermTag::from(0x0111100000000000u64) );
    // println!("{:?}", TermTag::from(0x1111100000000000u64) );
    println!("{:0x}", 0b011);
    let c1 = Cell::new(SymbolId(2), Some(TermId(22)), Some(TermId(25)));
    println!("{:?}", c1.get_tag());
    println!("{:?}", c1.get_symbol_id());
    println!("{:?}", c1.get_port_0());
    println!("{:?}", c1.get_port_1());
    let v1 = Var::new(Some(Polarity::Pos), ValueId(23));
    println!("{:0b}", ValueId(23).to_Instr());
    println!("{:0b}", v1.to_Instr());
    println!("{:0b}", VAR_POLARITY);
    println!("- {:0b}", VAR_POLARITY * Some(Polarity::Pos).map(Polarity::to_Instr).unwrap_or(0));
    println!("- {:0x}", VAR_POLARITY * Some(Polarity::Pos).map(Polarity::to_Instr).unwrap_or(0));
    println!("{:?}", v1.get_tag());
    println!("{:?}", v1.get_polarity());
    println!("{:?}", v1.get_value_id());
}

pub const VAL: u64 = 1;
pub const EXT: u64 = 0x100000000;
pub const ARI: u64 = 0x100000000000000;
pub const TAG: u64 = 0x1000000000000000;
