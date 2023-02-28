
use std::{collections::HashMap, sync::Arc, hash::Hash};

use crate::net::{Polarity, Polarized};


/// ## Symbol
///

#[derive(Debug, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub polarity: Polarity,
    pub polarities: Vec<Polarity>
}

impl Symbol {
    fn new(name: String, polarity: Polarity, polarities: Vec<Polarity>) -> Self {
        Self { name: name.to_owned(), polarity, polarities}
    }

    pub fn arity(&self) -> usize {
        self.polarities.len()
    }
}

impl Hash for Symbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
impl Polarized for Symbol {
    fn polarity(&self) -> &Polarity {
        &self.polarity
    }
}

#[derive(Debug)]
pub struct PortRef {
    pub symbol: Arc<Symbol>,
    pub port: u8
}

/// ## SymbolBook
///
#[derive(Debug)]
pub struct SymbolBook {
    symbols: HashMap<String, Arc<Symbol>>
}

impl SymbolBook {
    pub fn new(action: fn(&mut SymbolBuilder)) -> Arc<Self> {
        let mut symbols = HashMap::new();
        let mut builder = SymbolBuilder::new(&mut symbols);
        action(&mut builder);
        Arc::new(Self { symbols })
    }

    pub fn add(&mut self, name: &str, polarity: Polarity, polarities: Vec<Polarity>) {
    }

    pub fn find(&self, name: &str) -> Option<&Arc<Symbol>> {
        self.symbols.get(name.clone())
    }
}

pub struct SymbolBuilder<'a> {
    symbols: &'a mut HashMap<String, Arc<Symbol>>
}

impl<'a> SymbolBuilder<'a> {
    fn new(symbols: &'a mut HashMap<String, Arc<Symbol>>) -> Self {
        Self { symbols }
    }

    pub fn add(&mut self, name: &str, polarity: Polarity, polarities: Vec<Polarity>){
        let symbol = Symbol::new(name.to_string(), polarity, polarities);
        self.symbols.insert(symbol.name.clone(), Arc::new(symbol));
    }
}