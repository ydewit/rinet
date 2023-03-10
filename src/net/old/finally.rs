use std::{fmt::{Display, Debug}, marker::PhantomData, cell::Cell};

use super::Polarity;


#[derive(Debug)]
#[repr(packed)]
struct CompactSymbolPtr(u64);
impl CompactSymbolPtr {
    fn new(is_ctr: bool, arity: u8) -> Self{
        // assert!(index <= !0xF);
        // let mut val = index as u64;

        assert!(arity <= 0b111);
        let mut val = (arity as u64) << 61 & 0b11;

        if is_ctr {
            val |= 1 << 63 & 0b1;
        }
        else {
            val |= 0 << 63 & 0b1;
        }

        CompactSymbolPtr(val)
    }

    fn is_ctr(&self) -> bool {
        (self.0 >> 63 & 0b1) == 0b1
    }

    fn arity(&self) -> u8 {
        (self.0 >> 61 & 0b11) as u8
    }
}
impl Symbol for Id<CompactSymbolPtr> {
    fn get_polarity(&self) -> Polarity {
        match self.meta.is_ctr() {
            true => Polarity::Pos,
            false => Polarity::Neg,
        }
    }
}




// ---------------------------------------------------------------------------


// ---------------------------------------------------------------------------





// #[derive(Debug)]
// struct CellPtr(u64);

// #[derive(Debug)]
// struct VarPtr(u64);
// impl VarPtr {
//     fn bvar(name: &str) -> Self {
//         Self { free: false, name: name.to_string() }
//     }

//     fn fvar(name: &str) -> Self {
//         Self { free: true, name: name.to_string() }
//     }
// }

// const FIELD_TAG : BitField<TermTag> = BitField::new(0b10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000, 0b1);
// const FIELD_SYMBOL_ID : BitField<SymbolIdR> = BitField::new(0b00000000_10000000_00000000_00000000_00000000_00000000_00000000_00000000, 0b01111111_1111);
// const FIELD_PORT_0    : BitField<TermIdR> = BitField::new(0b00000000_00000000_00000000_00000000_00000001_00000000_00000000_00000000, 0b00000000_00001111_11111111_11111111_1111);
// const FIELD_PORT_1    : BitField<TermIdR> = BitField::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001, 0b00000000_00000000_00000000_00000000_00001111_11111111_11111111_11111111);

// const FIELD_POLARITY : BitField<Polarity> = BitField::new(0b00100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000, 0b011);
// const FIELD_VALUE_ID : BitField<ValueId> = BitField::new(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001, 0b00000000_00000000_00000000_00000000_00000000_11111111_11111111_11111111);

// mod test {
//     use std::collections::HashMap;

//     use super::*;

//     struct SymbolId(usize);
//     struct Symbol {
//         name: String,
//         polarity: Polarity,
//         polarities: Vec<Polarity>
//     }

//     trait TermId {}

//     struct CellId(usize);
//     impl TermId for CellId {}

//     struct Cell {
//         symbol_id: SymbolId,
//         ports: Vec<Box<dyn TermId>>,
//     }


//     struct VarId(usize);
//     impl TermId for VarId {}

//     struct Var {
//         name: String
//     }

//     #[derive(Debug)]
//     struct Print {

//     }

//     impl Default for Print {
//         fn default() -> Self {
//             Self {}
//         }
//     }

//     impl TermIdR for String {}
//     impl SymbolIdR for String {}

//     impl Net for Print {
//         type DataIdR = String;
//         type Fun = String;

//         type Equation;

//         type Cell = String;
//         type VarR = String;

//         fn data0(&mut self, name: &str) -> Self::DataIdR {
//             format!("+{name}")
//         }

//         fn data1(&mut self, name: &str, port: Polarity) -> Self::DataIdR {
//             format!("(+{name} {port})")
//         }

//         fn data2(&mut self, name: &str, port_0: Polarity, port_1: Polarity) -> Self::DataIdR {
//             format!("(+{name} {port_0} {port_1})")
//         }

//         fn get_data(&self, name: &str) -> Option<Self::DataIdR> {
//             None
//         }

//         fn fun0(&mut self, name: &str) -> Self::Fun {
//             format!("-{name}")
//         }

//         fn fun1(&mut self, name: &str, port: Polarity) -> Self::Fun {
//             format!("(-{name} {port})")
//         }

//         fn fun2(&mut self, name: &str, port_0: Polarity, port_1: Polarity) -> Self::Fun {
//             format!("(-{name} {port_0} {port_1})")
//         }

//         fn get_fun(&self, name: &str) -> Option<Self::Fun> {
//             None
//         }

//         fn cell0<S>(&mut self, symbol: &S) -> Self::Cell
//                 where S: SymbolIdR {
//             format!("({symbol})")
//         }

//         fn cell1<S, T>(&mut self, symbol: &S, port: &T) -> Self::Cell
//                 where S: SymbolIdR, T: TermIdR {
//             format!("({symbol} {port})")
//         }

//         fn cell2<S, T0, T1>(&mut self, symbol: &S, port_0: &T0, port_1: &T1) -> Self::Cell
//                 where S: SymbolIdR, T0: TermIdR, T1: TermIdR {
//             format!("({symbol} {port_0} {port_1})")
//         }

//         fn fvar(&mut self, name: &str, polarity: Polarity) -> Self::Var {
//             format!("[{name}]")
//         }

//         fn bvar(&mut self, name: &str) -> Self::Var {
//             format!("\\{{name}}")
//         }

//         fn redex(&mut self, output: &Self::Cell, input: &Self::Cell) {
//             format!("{output} == {input}");
//         }

//         fn bind<P: IO>(&mut self, var: &Self::Var, cell: &Self::Cell){
//             format!("{var} <- {cell}");
//         }

//         fn connect(&mut self, left: &Self::Var, right: &Self::Var) {
//             format!("{left} ↔ {right}");
//         }
//     }


//     fn prog<S : Net>() -> S {
//         let mut net = S::default();

//         let zero = net.data0("Z");
//         let one = net.data1("S", Polarity::Neg);
//         let add = net.fun2("add", Polarity::Pos, Polarity::Neg);

//         let r_fvar = net.fvar("r", Polarity::Neg);

//         let zero_cell = net.cell0(&zero);
//         let one_cell = net.cell1(&one , &zero_cell);
//         let add_cell = net.cell2(&add, &r_fvar, &zero_cell);


//         net.redex(&one_cell, &add_cell);
//         net
//     }

//     #[test]
//     fn test() {
//         println!("{:?}", prog::<Print>());
//     }
// }