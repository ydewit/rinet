mod compact;
pub enum SymbolArity {
    Zero = 0,
    One = 1,
    Two = 2,
}

pub enum SymbolKind {
    Fun,
    Ctr,
}

pub enum Polarity {
    Pos,
    Neg,
}

impl From<SymbolKind> for Polarity {
    fn from(kind: SymbolKind) -> Self {
        match kind {
            SymbolKind::Fun => Polarity::Neg,
            SymbolKind::Ctr => Polarity::Pos,
        }
    }
}
pub trait SymbolSym {
    type Symbol;

    fn symbol0(name: &str, kind: SymbolKind) -> Self::Symbol;
    fn symbol1(name: &str, kind: SymbolKind, port_polarity: Polarity) -> Self::Symbol;
    fn symbol2(
        name: &str,
        kind: SymbolKind,
        left_polarity: Polarity,
        right_polarity: Polarity,
    ) -> Self::Symbol;
}
