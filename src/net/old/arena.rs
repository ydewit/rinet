///! Arena based implementation of interaction nets separating a generational Id from the actual objects stored
///! in a plain vector.
///!

use std::{fmt::Debug, sync::OnceLock, marker::PhantomData, collections::HashMap, env::var, hash::{Hash, Hasher}, ops::{Index, IndexMut}};

use super::Polarity;



/// ## symbol
///
type SymbolId = ArenaId<Symbol, (Polarity,u8)>;

#[derive(Debug, Eq)]
struct Symbol {
    name: String,
    polarity: Polarity,
    polarities: Vec<Polarity>
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
    fn ne(&self, other: &Self) -> bool {
        self.name.ne(&other.name)
    }
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl Symbol {
    pub fn new(name: &str, polarities: Vec<Polarity>) -> Self {
        Self { name: name.to_string(), polarity: Symbol::to_polarity(name), polarities }
    }

    fn to_polarity(name: &str) -> Polarity {
        if name.chars().next().unwrap().is_uppercase() {
            return Polarity::Pos
        }
        else {
            return Polarity::Neg
        }
    }
}

// /// ## SymbolBook
// ///
// #[derive(Debug)]
// pub struct SymbolBook {
//     symbols: HashMap<String, SymbolId>
// }

// impl SymbolBook {
//     pub fn new(action: fn(&mut SymbolBuilder)) -> Self {
//         let mut symbols = HashMap::new();
//         let mut builder = SymbolBuilder::new(&mut symbols);
//         action(&mut builder);
//         Self { symbols }
//     }

//     pub fn add(&mut self, name: &str, polarity: Polarity, polarities: Vec<Polarity>) {
//     }

//     pub fn find(&self, name: &str) -> Option<&Arc<Symbol>> {
//         self.symbols.get(name.clone())
//     }
// }

// pub struct SymbolBuilder<'a> {
//     symbols: &'a mut HashMap<String, Arc<Symbol>>
// }

// impl<'a> SymbolBuilder<'a> {
//     fn new(symbols: &'a mut HashMap<String, Arc<Symbol>>) -> Self {
//         Self { symbols }
//     }

//     pub fn add(&mut self, name: &str, polarity: Polarity, polarities: Vec<Polarity>){
//         let symbol = Symbol::new(name.to_string(), polarities);
//         self.symbols.insert(symbol.name.clone(), );
//     }
// }

/// ## NetFamily
///
// trait Allocator {
//     type Id: Copy + Debug;
//     fn alloc(&mut self, value: T) -> Self::Id;
//     fn alloc_with<C: FnOnce() -> T>(&mut self, create_fn: C) -> ArenaId<T>;
//     fn get(&self, id: &Self::Id) -> Option<&T>;
//     fn free(&mut self, id: &Self::Id) -> Option<T>;
// }

pub trait NetFamily : Debug {
    // type Id<T: Debug> : Debug;
    type FVarVal : Debug;
    type BVarVal : Debug;
}


/// ## TermFamily
#[derive(Debug)]
pub struct TermFamily {}
impl NetFamily for TermFamily {
    // type Id<T: Debug> = ArenaId<T>;
    type FVarVal = OnceLock<CellId>;
    type BVarVal= OnceLock<CellId>;
}

/// ## FVar
///
#[derive(Debug)]
struct FVar<F: NetFamily = TermFamily>(F::FVarVal);
type FVarId<F: NetFamily = TermFamily> = ArenaId<FVar<F>>;

/// ## BVar
///
#[derive(Debug)]
struct BVar<F: NetFamily = TermFamily>(F::BVarVal);
type BVarId<F: NetFamily = TermFamily> = ArenaId<BVar<F>>;


/// ## VarId
///
#[derive(Debug)]
enum VarId<F: NetFamily = TermFamily> {
    FVar(FVarId<F>),
    BVar(BVarId<F>)
}


/// ## Cell
///
#[derive(Debug)]
struct Cell<F: NetFamily = TermFamily> {
    symbol_id: SymbolId,
    polarity: Polarity,
    ports: Vec<TermId<F>>
}
type CellId<F: NetFamily = TermFamily> = ArenaId<Cell<F>, Polarity>;

impl<F: NetFamily> Cell<F> {
    pub fn new(symbol_id: SymbolId, ports: Vec<TermId<F>>) -> Self {
        let polarity = symbol_id.metadata.0;
        Self {symbol_id, polarity, ports}
    }

    pub fn alloc(cells: &mut Arena<Cell<F>>, symbol_id: SymbolId, ports: Vec<TermId<F>>) -> CellId<F> {
        let polarity = symbol_id.metadata.0;
        cells.alloc(Cell::new(symbol_id, ports), polarity)
    }
}


/// ## TermId
#[derive(Debug)]
enum TermId<F: NetFamily = TermFamily> {
    Cell(CellId<F>),
    FVar(FVarId<F>),
    BVar(BVarId<F>)
}

impl<F: NetFamily> From<FVarId<F>> for TermId<F> {
    fn from(value: FVarId<F>) -> Self {
        TermId::FVar(value)
    }
}

impl<F: NetFamily> From<BVarId<F>> for TermId<F> {
    fn from(value: BVarId<F>) -> Self {
        TermId::BVar(value)
    }
}

impl<F: NetFamily> From<CellId<F>> for TermId<F> {
    fn from(value: CellId<F>) -> Self {
        TermId::Cell(value)
    }
}

impl<F: NetFamily> From<VarId<F>> for TermId<F> {
    fn from(var_id: VarId<F>) -> Self {
        match var_id {
            VarId::FVar(value) => TermId::FVar(value),
            VarId::BVar(value) => TermId::BVar(value),
        }

    }
}

/// ## EquationId
#[derive(Debug)]
enum Equation<F: NetFamily = TermFamily> {
    Redex { lhs_id: CellId<F>, rhs_id: CellId<F> },
    Bind { var_id: VarId<F>, cell_id: CellId<F> },
    Connect { lhs_id: VarId<F>, rhs_id: VarId<F> }
}
type EquationId<F: NetFamily = TermFamily> = ArenaId<Equation<F>>;

/// ## SymbolBook
///
#[derive(Debug)]
struct SymbolBook {
    names: HashMap<String, SymbolId>,
    symbols: Arena<Symbol>
}

impl SymbolBook {
    fn new() -> Self {
        Self { names: HashMap::new(), symbols: Arena::new() }
    }

    pub fn add(&mut self, name: &str, polarities: Vec<Polarity>) -> SymbolId {
        let len = polarities.len();
        let symbol = Symbol::new(name, polarities);
        let polarity = symbol.polarity;
        let id = self.symbols.alloc(symbol, (polarity, len as u8));
        self.names.insert(name.to_string(), id);
        id.clone()
    }

    pub fn get(&self, symbol_id: &SymbolId) -> Option<&Symbol> {
        self.symbols.get(symbol_id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&SymbolId> {
        self.names.get(name)
    }
}

/// ## Net
///
#[derive(Debug)]
struct Net<F: NetFamily = TermFamily> {
    head: Vec<FVarId<F>>,
    body: Vec<EquationId<F>>,

    equations: Arena<Equation<F>>,
    cells: Arena<Cell<F>>,
    fvars: Arena<FVar<F>>,
    bvars: Arena<BVar<F>>,
}

impl Net {
    pub fn build<C>(symbols: &SymbolBook, create_fn: C) -> Self
        where C: FnOnce(&mut NetBuilder) {
        let mut builder = NetBuilder::new(symbols);
        create_fn(&mut builder);
        let net = builder.build();
        net
    }
}

impl<F: NetFamily> Net<F> {
    fn new() -> Self {
        Self {
            head: Vec::new(),
            body: Vec::new(),
            equations: Arena::new(),
            cells: Arena::new(),
            fvars: Arena::new(),
            bvars: Arena::new()
        }
    }

    pub fn get_cell(&self, id: &CellId<F>) -> Option<&Cell<F>> {
        self.cells.get(id)
    }

    pub fn get_bvar(&self, id: &BVarId<F>) -> Option<&BVar<F>> {
        self.bvars.get(id)
    }

    pub fn get_fvar(&self, id: &FVarId<F>) -> Option<&FVar<F>> {
        self.fvars.get(id)
    }

    // pub fn get_var(&self, id: &VarId<F>) -> Either<&FVar<F>, &BFVar<F>> {
    //     self.fvars.get(id)
    // }

    pub fn get_equation(&self, id: &EquationId<F>) -> Option<&Equation<F>> {
        self.equations.get(id)
    }
}

struct NetBuilder<'a> {
    symbols: &'a SymbolBook,
    net: Net
}

impl<'a> NetBuilder<'a> {
    pub fn new(symbols: &'a SymbolBook) -> Self {
        Self { symbols, net: Net::new() }
    }

    // fn get_symbol(&self, &)
    fn cell(&mut self, symbol_id: SymbolId, ports: Vec<TermId>) -> CellId {
        Cell::alloc(&mut self.net.cells, symbol_id, ports)
    }

    fn fvar(&mut self) -> FVarId {
        let value = OnceLock::new();
        self.net.fvars.alloc(FVar(value), ())
    }

    fn bvar(&mut self) -> BVarId {
        let value = OnceLock::new();
        self.net.bvars.alloc(BVar(value), ())
    }

    fn bind(&mut self, var_id: VarId, cell_id: CellId) -> EquationId {
        self.net.equations.alloc(Equation::Bind{var_id, cell_id}, ())
    }

    fn redex(&mut self, lhs_id: CellId, rhs_id: CellId) -> EquationId {
        self.net.equations.alloc(Equation::Redex{lhs_id, rhs_id}, ())
    }

    fn connect(&mut self, lhs_id: VarId, rhs_id: VarId) -> EquationId {
        self.net.equations.alloc(Equation::Connect{lhs_id, rhs_id}, ())
    }

    fn build(&self) -> Net {
        Net {
            head: Vec::new(),
            body: Vec::new(),
            equations: Arena::new(),
            cells: Arena::new(),
            fvars: Arena::new(),
            bvars: Arena::new()
        }
    }
}
/// ## Rule
///
#[derive(Debug)]
struct RuleFamily;

impl NetFamily for RuleFamily {
    // type Id<T: Debug> = ArenaId<T>;
    type BVarVal = char; // e.g. 'x', 'y' etc
    type FVarVal = (SymbolId, u8);
}

type RuleId = ArenaId<Rule>;
type RuleNet = Net<RuleFamily>;
type RuleFVar = FVar<RuleFamily>;
type RuleBVar = BVar<RuleFamily>;
type RuleFVarId = FVarId<RuleFamily>;
type RuleBVarId = BVarId<RuleFamily>;
type RuleVarId = VarId<RuleFamily>;
type RuleCellId = CellId<RuleFamily>;
type RuleTermId = TermId<RuleFamily>;

#[derive(Debug)]
struct Rule {
    out_symbol_id: SymbolId,
    in_symbol_id: SymbolId,
    body: RuleNet
}


impl Rule {
    fn reduce(&self, net: &mut Net, output: &Cell, input: &Cell) {
        assert!(output.polarity == Polarity::Pos && input.polarity == Polarity::Neg);
        todo!()
    }
}

/// ## RuleBuilder
///
struct RuleBuilder {
    out_symbol_id: SymbolId,
    in_symbol_id: SymbolId,
    body: Net<RuleFamily>,
}

impl RuleBuilder {
    pub fn new(out_symbol_id: SymbolId, in_symbol_id: SymbolId) -> Self {
        assert!(out_symbol_id.metadata.0 == Polarity::Pos && in_symbol_id.metadata.0 == Polarity::Neg);

        let mut body = RuleNet::new();

        Self { out_symbol_id, in_symbol_id, body }
    }

    fn out_fvar(&mut self, port: u8) -> RuleVarId {
        let fvar_id = self.body.fvars.alloc(FVar((self.out_symbol_id, port)), ());
        RuleVarId::FVar(fvar_id)
    }

    fn in_fvar(&mut self, port: u8) -> RuleVarId {
        let fvar_id = self.body.fvars.alloc(FVar((self.in_symbol_id, port)), ());
        RuleVarId::FVar(fvar_id)
    }

    pub fn bvar_term(&mut self, value: <RuleFamily as NetFamily>::BVarVal) -> RuleTermId {
        let bvar_id = self.body.bvars.alloc(BVar(value), ());
        RuleTermId::BVar(bvar_id)
    }

    pub fn bvar(&mut self, value: <RuleFamily as NetFamily>::BVarVal) -> RuleBVarId {
        self.body.bvars.alloc(BVar(value), ())
    }

    pub fn cell_term(&mut self, symbol_id: SymbolId, ports: Vec<RuleTermId>) -> RuleTermId {
        let cell_id = self.cell(symbol_id, ports);
        RuleTermId::Cell(cell_id)
    }

    pub fn cell(&mut self, symbol_id: SymbolId, ports: Vec<RuleTermId>) -> RuleCellId {
        Cell::alloc(&mut self.body.cells, symbol_id, ports)
    }

    pub fn connect(&mut self, lhs_id: RuleVarId, rhs_id: RuleVarId) -> EquationId<RuleFamily> {
        let eqn = Equation::Connect { lhs_id, rhs_id };
        self.body.equations.alloc(eqn, ())
    }

    pub fn bind(&mut self, var_id: RuleVarId, cell_id: RuleCellId) -> EquationId<RuleFamily> {
        let eqn = Equation::Bind { var_id, cell_id };
        self.body.equations.alloc(eqn, ())
    }

    pub fn redex(&mut self, lhs_id: RuleCellId, rhs_id: RuleCellId) -> EquationId<RuleFamily> {
        let eqn = Equation::Redex { lhs_id, rhs_id };
        self.body.equations.alloc(eqn, ())
    }

    fn build(self) -> Rule {
        let body = self.body;
        Rule {
            out_symbol_id: self.out_symbol_id,
            in_symbol_id: self.in_symbol_id,
            body
        }
    }
}

/// ## RuleBook
///
#[derive(Debug)]
struct RuleBook {
    rule_id_by_symbols: HashMap<(SymbolId, SymbolId), RuleId>,
    rules: Arena<Rule>
}

impl RuleBook {
    fn new() -> Self {
        Self { rule_id_by_symbols: HashMap::new(), rules: Arena::new() }
    }

    pub fn add<F>(&mut self, out_symbol_id: SymbolId, in_symbol_id: SymbolId, create_fn: F)
        where F: FnOnce(&mut RuleBuilder, &[RuleVarId], &[RuleVarId]) {
        assert!(out_symbol_id.metadata.0 == Polarity::Pos && in_symbol_id.metadata.0 == Polarity::Neg);
        let out_len = out_symbol_id.metadata.1 as usize;
        let in_len = in_symbol_id.metadata.1 as usize;

        let mut builder = RuleBuilder::new(out_symbol_id, in_symbol_id);

        let mut out_fvars: Vec<RuleVarId> = Vec::with_capacity(out_len);
        for o in 0..out_len {
            let fvar = builder.body.fvars.alloc(FVar((out_symbol_id, o as u8)), ());
            out_fvars[o] = RuleVarId::FVar(fvar);
        }

        let mut in_fvars: Vec<RuleVarId> = Vec::with_capacity(in_len);
        for i in 0..in_len {
            let fvar = builder.body.fvars.alloc(FVar((in_symbol_id, i as u8)), ());
            in_fvars[i] = RuleVarId::FVar(fvar);
        }

        create_fn(&mut builder, &out_fvars, &in_fvars);
        let rule = builder.build();

        let rule_id = self.rules.alloc(rule, ());
        self.rule_id_by_symbols.insert((out_symbol_id, in_symbol_id), rule_id);
    }

    pub fn get(&self, lhs_id: &SymbolId, rhs_id: &SymbolId) -> Option<&Rule> {
        let symbol_ids = &(*lhs_id, *rhs_id); // TODO can we avoid the copy??
        match self.rule_id_by_symbols.get(symbol_ids) {
            Some(rule_id) => self.rules.get(&rule_id),
            None => None,
        }
    }
}


/// ## Arena
///
const DEFAULT_ENTRY_CAPACITY: usize = 100;
const DEFAULT_FREE_CAPACITY: usize = 20;

#[derive(Clone, Debug, PartialEq, Eq)]
enum ValueEntry<V> {
    Occupied { value: V, generation: u64 },
    Free { free_index: usize }
}


#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct Arena<T> {
    entries: Vec<ValueEntry<T>>,
    free: Vec<ArenaId<T>>,
}

impl<T> Arena<T> {
    #[inline]
    pub fn new() -> Arena<T> {
        Arena::with_capacity(DEFAULT_ENTRY_CAPACITY, DEFAULT_FREE_CAPACITY)
    }

    #[inline]
    pub fn with_capacity(entries_capacity: usize, free_capacity: usize) -> Self {
        Self { entries: Vec::with_capacity(entries_capacity), free: Vec::with_capacity(free_capacity) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len() - self.free.len()
    }

    #[inline]
    pub fn alloc<M: Copy>(&mut self, value: T, metadata: M) -> ArenaId<T, M> {
        let index = self.entries.len();
        self.entries.push( ValueEntry::Occupied { value: value, generation: 0 } );
        ArenaId::new(index, 0, metadata)
    }

    #[inline]
    pub fn alloc_with<M: Copy, C: FnOnce() -> T>(&mut self, create_fn: C, metadata: M) -> ArenaId<T, M> {
        let index = self.entries.len();
        self.entries.push(ValueEntry::Occupied { value: create_fn(), generation: 0 });
        ArenaId::new(index, 0, metadata)
    }

    #[inline]
    pub fn get<M: Copy>(&self, id: &ArenaId<T, M>) -> Option<&T> {
        match self.entries.get(id.index) {
            Some(entry) => {
                match entry {
                    ValueEntry::Occupied { value, generation } => Some(value),
                    ValueEntry::Free { free_index } => None,
                }
            },
            None => None,
        }
    }

    pub fn remove(&mut self, id: &ArenaId<T>) -> Option<T> {
        if id.index >= self.entries.len() {
            return None
        }

        let entry = &self.entries[id.index];
        if let ValueEntry::Free { free_index } = entry {
            return None
        }

        let free_index = self.free.len();
        let old_entry = std::mem::replace(&mut self.entries[id.index], ValueEntry::Free { free_index });
        match old_entry {
            ValueEntry::Occupied { value, .. } => Some(value),
            ValueEntry::Free { .. } => None,
        }
    }

    fn is_occupied(&self, entry: &ValueEntry<T>) -> bool {
        match entry {
            ValueEntry::Occupied { .. } => true,
            _ => false
        }
    }

    // pub fn iter(&self) -> Iter<F::Id> {
    //     self.entries.iter().filter(|e|{self.is_occupied(e)}).enumerate().map(|(index, entry)|{
    //         match entry {
    //             ValueEntry::Occupied { value, generation } => ArenaId::new(index, *generation),
    //             _ => panic!("Unexpected")
    //         }
    //     })
    // }
}

/// ## NetRuntime
///
#[derive(Debug)]
struct NetRuntime<'a> {
    symbols: &'a SymbolBook,
    rules: &'a RuleBook
}

impl<'a> NetRuntime<'a> {
    pub fn new(symbols: &'a SymbolBook, rules: &'a RuleBook) -> Self {
        Self {
            symbols,
            rules
        }
    }

    pub fn eval(&self, net: Net) {
        while let Some(eqn_id) = net.body.pop() {
            self.eval_equation(&mut net, &eqn_id);
        }
    }

    fn eval_equation(&self, net: &mut Net, eqn_id: &EquationId) {
        let eqn = net.equations.remove(eqn_id);
        match eqn {
            Some(Equation::Redex { lhs_id, rhs_id }) => {
                // match (lhs_id.metadata, rhs_id.metadata) {
                //     (Polarity::Pos, Polarity::Neg) => self.eval_redex(&mut net, lhs_id, rhs_id),
                //     (Polarity::Neg, Polarity::Pos) => self.eval_redex(&mut net, rhs_id, lhs_id),
                //     _ => panic!("Pos <> Neg or Neg <> Pos only!")
                // }
            },
            Some(Equation::Bind { var_id, cell_id }) => {
                // match var_id {
                //     VarId::FVar(fvar_id) => {
                //         self.bind_fvar(&mut net, fvar_id, cell_id);
                //     },
                //     VarId::BVar(bvar_id) => {
                //         self.bind_bvar(&mut net, bvar_id, cell_id);
                //     }
                // }
            },
            Some(Equation::Connect { lhs_id, rhs_id }) => {
                self.eval_connect(&mut net, lhs_id, lhs_id);
            },
            None => panic!("Equation not found"),
        }
    }

    fn eval_redex(&self, net: &mut Net, output_id: &CellId, input_id: &CellId) {
        let output = net.cells.get(&output_id).unwrap();
        let input = net.cells.get(&input_id).unwrap();
        match self.rules.get(&output.symbol_id, &input.symbol_id) {
            Some(rule) => {
                rule.reduce(net, output, input)
            },
            None => panic!("Rule not found: {:?} <> {:?}", &output.symbol_id, &input.symbol_id),
        }
    }

    /// Bind a cell to a free-variable
    ///
    /// If the cell is positive (i.e. a constructor), then the net will set the value and assume
    /// that a consumer will get it either in real time or batch.
    fn bind_fvar(&self, net: &mut Net, fvar_id: &FVarId, cell_id: &CellId) {
        match net.fvars.get(&fvar_id) {
            Some(FVar(val)) => {
                match cell_id.metadata {
                    Polarity::Pos => {
                        val.set(*cell_id);
                    },
                    Polarity::Neg => {
                        val.get(); // TODO
                    }
                }
            },
            None => panic!("FVar not found: {:?}", fvar_id),
        }
    }

    /// Bind a cell to a bounded-var
    ///
    /// Bounded vars are set and get within the Net so it needs to handle
    /// both cases.
    fn bind_bvar(&self, net: &mut Net, bvar_id: &BVarId, cell_id: &CellId) {
        match net.bvars.get(&bvar_id) {
            Some(BVar(val)) => {
                let other_cell_id = val.get_or_init(||{*cell_id}); // TODO can we safely avoid a copy here?
                match cell_id.metadata {
                    Polarity::Pos => {
                        if other_cell_id.metadata == Polarity::Neg {
                            net.bvars.remove(bvar_id);
                            self.eval_redex(net, cell_id, other_cell_id)
                        }
                        // otherwise, we wait on the other cell
                    },
                    Polarity::Neg => {
                        if other_cell_id.metadata == Polarity::Pos {
                            net.bvars.remove(bvar_id);
                            self.eval_redex(net, other_cell_id, cell_id)
                        }
                        // otherwise, we wait on the other cell
                    },
                }
            },
            None => todo!(),
        }
    }

    fn eval_connect(&self, net: &mut Net, lhs_id: &VarId, rhs_id: &VarId) {
        match (lhs_id, rhs_id) {
            (VarId::FVar(_), VarId::FVar(_)) => todo!(),
            (VarId::FVar(_), VarId::BVar(_)) => todo!(),
            (VarId::BVar(_), VarId::FVar(_)) => todo!(),
            (VarId::BVar(_), VarId::BVar(_)) => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct ArenaId<T, M : Copy = ()> {
    index: usize,
    generation: u64,
    metadata: M,
    _t: PhantomData<fn() -> T>
}

impl<T, M: Copy> ArenaId<T, M> {
    pub fn new(index: usize, generation: u64, metadata: M) -> Self {
        Self { index, generation, metadata, _t: PhantomData }
    }
}

impl<T, M: Copy> Clone for ArenaId<T, M> {
    fn clone(&self) -> Self {
        ArenaId::new(self.index, self.generation, self.metadata)
    }
}

impl<T, M: Copy> Copy for ArenaId<T, M> {
}
/// ## NetEngine
///





#[cfg(test)]
mod tests {
    use super::*;

    use Polarity::*;
    #[test]
    fn t() {
        let mut symbols = SymbolBook::new();
        let z_symbol_id = symbols.add("Z", vec![]);
        let s_symbol_id = symbols.add("S", vec![Neg]);
        let add_symbol_id = symbols.add("add", vec![Pos, Neg]);

        // ----- RULES

        let mut rules = RuleBook::new();

        // add >< Z ⟶
        rules.add(add_symbol_id, z_symbol_id, |b, z, add| {
            b.connect(add[0], add[0]);
        });

        // add >< S ⟶
        rules.add(add_symbol_id, z_symbol_id, |b, s, add| {
            let x = b.bvar('x');
            b.bind(s[0], b.cell(add_symbol_id, vec![TermId::from(x), TermId::from(add[1])]));
            b.bind(add[0], b.cell(s_symbol_id, vec![TermId::from(x)]));
        });


        // ----- NET

        let mut net = Net::build(&symbols, |b|{
            let r = b.fvar();
            let z_cell = b.cell(z_symbol_id, vec![]);
            let add_cell = b.cell(add_symbol_id, vec![TermId::from(r), TermId::from(z_cell)]);
            let s_cell = b.cell(s_symbol_id, vec![TermId::from(z_cell)]);
            b.redex(add_cell, s_cell);
        });


        // ---- NetRuntime
        let runtime = NetRuntime::new(&symbols, &rules);
        runtime.eval(&mut net);
    }

    // #[test]
    // fn test_arena() {
    //     let mut arena = Arena::<String>::new();
    //     let id = arena.alloc("aaa".to_string());
    //     println!("Id = {:?}", id);

    //     println!("Get Id: {:?}", arena.get(&id));
    //     let val = arena.remove(&id);
    //     println!("Remove Id: {:?}", val);
    //     let val = arena.remove(&id);
    //     println!("Remove Id: {:?}", val);

    //     println!("Get Id: {:?}", arena.get(&id));
    // }
}
