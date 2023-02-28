use super::{arena::{Arena, Ptr, ToTag}, Polarity, dsl::{SymbolDsl}, Polarized};

#[derive(Debug,Clone,Copy)]
pub enum SymbolTag {
    Ctr { arity: u8 },
    Fun { arity: u8 }
}
impl ToTag<SymbolTag> for SymbolEntry {
    fn to_tag(&self) -> SymbolTag {
        match self.polarity {
            Polarity::Pos => SymbolTag::Ctr { arity: self.arity() },
            Polarity::Neg => SymbolTag::Fun { arity: self.arity() },
        }
    }
}

impl Polarized for SymbolTag {
    fn polarity(&self) -> Polarity {
        match self {
            SymbolTag::Ctr { .. } => Polarity::Pos,
            SymbolTag::Fun { .. } => Polarity::Neg,
        }
    }
}

#[derive(Debug)]
struct SymbolEntry {
    name: String,
    polarity: Polarity,
    polarities: Vec<Polarity>
}

impl Polarized for SymbolEntry {
    fn polarity(&self) -> Polarity {
        self.polarity
    }
}

impl SymbolEntry {
    fn new0(name: &str,  polarity: Polarity) -> Self {
        SymbolEntry { name: name.to_string(), polarity, polarities: Vec::new() }
    }

    fn new1(name: &str,  polarity: Polarity, port: Polarity) -> Self {
        SymbolEntry { name: name.to_string(), polarity, polarities: vec![port] }
    }

    fn new2(name: &str, polarity: Polarity, port_0: Polarity, port_1: Polarity) -> Self {
        SymbolEntry { name: name.to_string(), polarity, polarities: vec![port_0, port_1] }
    }

    fn arity(&self) -> u8 {
        self.polarities.len() as u8
    }
}


#[derive(Debug)]
pub struct SymbolBook {
    symbols: Arena<SymbolEntry, SymbolTag>
}

impl SymbolBook {
    fn alloc(&mut self, symbol: SymbolEntry) -> Ptr<SymbolTag> {
        let arity = symbol.arity();
        let tag = match symbol.polarity {
            Polarity::Pos => SymbolTag::Ctr { arity },
            Polarity::Neg => SymbolTag::Fun { arity }
        };
        self.symbols.alloc(symbol, tag)
    }

    fn free(&mut self, ptr: Ptr<SymbolTag>) -> Option<SymbolEntry> {
        self.symbols.free(ptr)
    }
}

impl Default for SymbolBook {
    fn default() -> Self {
        Self { symbols: Arena::new() }
    }
}

impl SymbolDsl for SymbolBook {
    type CtrSymbolRef =  Ptr<SymbolTag>;
    type FunSymbolRef = Ptr<SymbolTag>;

    fn ctr0(&mut self, name: &str) -> Self::CtrSymbolRef {
        self.alloc(SymbolEntry::new0(name, Polarity::Pos))
    }

    fn ctr1(&mut self, name: &str, port: Polarity) -> Self::CtrSymbolRef {
        self.alloc(SymbolEntry::new1(name, Polarity::Pos, port))
    }

    fn ctr2(&mut self, name: &str, port_0: Polarity, port_1: Polarity) -> Self::CtrSymbolRef {
        self.alloc(SymbolEntry::new2(name, Polarity::Pos, port_0, port_1))
    }

    fn fun0(&mut self, name: &str) -> Self::FunSymbolRef {
        self.alloc(SymbolEntry::new0(name, Polarity::Neg))
    }

    fn fun1(&mut self, name: &str, port: Polarity) -> Self::FunSymbolRef {
        self.alloc(SymbolEntry::new1(name, Polarity::Neg, port))
    }

    fn fun2(&mut self, name: &str, port_0: Polarity, port_1: Polarity) -> Self::FunSymbolRef {
        self.alloc(SymbolEntry::new2(name, Polarity::Neg, port_0, port_1))
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn adder_book<S : SymbolDsl>() -> S {
        let mut book = S::default();

        let zero = book.ctr0("Z");
        let one = book.ctr1("S", Polarity::Neg);
        let add = book.fun2("add", Polarity::Pos, Polarity::Neg);

        book
    }

    #[test]
    fn test() {
        println!("{:?}", adder_book::<SymbolBook>());
    }

}