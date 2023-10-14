use bitfield_struct::bitfield;
use std::collections::HashMap;

use super::{Polarity, SymbolArity, SymbolKind, SymbolSym};

pub struct SymbolVal {
    name: String,
    arity: SymbolArity,
    polarity: Polarity,
    left_polarity: Option<Polarity>,
    right_polarity: Option<Polarity>,
}

pub struct SymbolBook {
    symbols: Vec<SymbolVal>,
    symbol_by_name: HashMap<String, usize>,
    name_by_symbol: HashMap<usize, String>,
}

impl SymbolSym for SymbolBook {
    type Symbol = SymbolVal;

    fn symbol0(name: &str, kind: SymbolKind) -> Self::Symbol {
        SymbolVal {
            name: name.to_string(),
            arity: SymbolArity::Zero,
            polarity: match kind {
                SymbolKind::Fun => Polarity::Neg,
                SymbolKind::Ctr => Polarity::Pos,
            },
            left_polarity: None,
            right_polarity: None,
        }
    }

    fn symbol1(name: &str, kind: SymbolKind, port_polarity: Polarity) -> Self::Symbol {
        SymbolVal {
            name: name.to_string(),
            arity: SymbolArity::Zero,
            polarity: match kind {
                SymbolKind::Fun => Polarity::Neg,
                SymbolKind::Ctr => Polarity::Pos,
            },
            left_polarity: Some(port_polarity),
            right_polarity: None,
        }
    }

    fn symbol2(
        name: &str,
        kind: SymbolKind,
        left_polarity: Polarity,
        right_polarity: Polarity,
    ) -> Self::Symbol {
        SymbolVal {
            name: name.to_string(),
            arity: SymbolArity::Zero,
            polarity: kind.into(),
            left_polarity: Some(left_polarity),
            right_polarity: Some(right_polarity),
        }
    }
}
