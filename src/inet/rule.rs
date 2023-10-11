use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
};

use super::{
    arena::{Arena, ArenaPtr, ArenaValue},
    cell::CellPtr,
    equation::{Equation, EquationDisplay, EquationPtr, Equations},
    heap::{CellDisplay, Heap, VarDisplay},
    symbol::{Symbol, SymbolArity, SymbolBook, SymbolName, SymbolPtr},
    term::{TermFamily, TermPtr},
    var::{PVarPtr, Var, VarPtr},
    BitSet16, Polarity,
};

#[derive(Debug, Clone)]
pub struct RuleF {}
impl TermFamily for RuleF {
    type BoundStore = u8;
    type FreeStore = RulePort;

    fn display_store(
        f: &mut std::fmt::Formatter<'_>,
        symbols: &SymbolBook,
        heap: &Heap<Self>,
        var: &Var<RuleF>,
        index: usize,
    ) -> std::fmt::Result {
        match var {
            Var::Bound(store) => {
                write!(f, "?{}", store)
            }
            Var::Free(RulePort::Ctr(PortNum::Zero)) => {
                write!(f, "C₀")
            }
            Var::Free(RulePort::Ctr(PortNum::One)) => {
                write!(f, "C₁")
            }
            Var::Free(RulePort::Fun(PortNum::Zero)) => {
                write!(f, "F₀")
            }
            Var::Free(RulePort::Fun(PortNum::One)) => {
                write!(f, "F₁")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum PortNum {
    Zero = 0,
    One = 1,
}

impl PortNum {
    pub fn is_valid_port(&self, arity: SymbolArity) -> bool {
        match arity {
            SymbolArity::Zero => false,
            SymbolArity::One => *self == PortNum::Zero,
            SymbolArity::Two => true,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum RulePort {
    Ctr(PortNum),
    Fun(PortNum),
}

impl Default for RulePort {
    fn default() -> Self {
        RulePort::Ctr(PortNum::Zero)
    }
}

impl Into<Var<RuleF>> for RulePort {
    fn into(self) -> Var<RuleF> {
        Var::Free(self)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct RulePtr(u16);
impl RulePtr {
    const INDEX: BitSet16<14> = BitSet16 {
        mask: 0b00111111_11111111,
        offset: 0,
    };

    pub fn new(index: usize) -> Self {
        let mut ptr = Self(0);
        ptr.set_index(index);
        ptr
    }

    pub fn get_index(&self) -> usize {
        Self::INDEX.get(self.0) as usize
    }

    fn set_index(&mut self, index: usize) {
        self.0 = Self::INDEX.set(self.0, index as u16)
    }
}

impl ArenaPtr for RulePtr {
    fn get_index(&self) -> usize {
        self.get_index()
    }
}

impl Debug for RulePtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("RulePtr({:016b})", self.0);
        let mut b = f.debug_struct(&name);
        b.field("index", &self.get_index());
        b.finish()
    }
}

type RuleKey = (usize, usize);

#[derive(Debug)]
pub struct Rule {
    pub(crate) ctr_ptr: SymbolPtr,
    pub(crate) fun_ptr: SymbolPtr,
    fvar_ptrs: Vec<PVarPtr>,
    bvar_count: u8,
    pub body: Vec<EquationPtr>,
}
impl Rule {
    pub fn new(ctr_ptr: SymbolPtr, fun_ptr: SymbolPtr) -> Self {
        Self {
            ctr_ptr,
            fun_ptr,
            fvar_ptrs: Vec::new(),
            bvar_count: 0,
            body: Vec::new(),
        }
    }

    pub fn get_key(&self) -> RuleKey {
        (self.ctr_ptr.get_index(), self.fun_ptr.get_index())
    }

    pub fn get_bvar_count(&self) -> u8 {
        self.bvar_count
    }

    pub fn get_fvar_count(&self) -> usize {
        self.fvar_ptrs.len()
    }

    pub fn body(&self) -> std::slice::Iter<EquationPtr> {
        self.body.iter()
    }
}

impl ArenaValue<RulePtr> for Rule {
    fn to_ptr(&self, index: usize) -> RulePtr {
        RulePtr::new(index)
    }
}

pub type Rules = Arena<Rule, RulePtr>;

pub struct RuleBuilder<'a, 'b> {
    rules: &'b mut RuleSet<'a>,
    rule: Rule,
    ctr_symbol: Symbol,
    fun_symbol: Symbol,
}

impl<'a, 'b> RuleBuilder<'a, 'b> {
    fn new(ctr: SymbolPtr, fun: SymbolPtr, rules: &'b mut RuleSet<'a>) -> Self {
        let ctr_symbol = rules.symbols.get(ctr);
        let fun_symbol = rules.symbols.get(fun);
        Self {
            rules,
            rule: Rule::new(ctr, fun),
            ctr_symbol,
            fun_symbol,
        }
    }

    fn build(self) -> RulePtr {
        let rule_key = self.rule.get_key();

        let rule_ptr = self.rules.rules.alloc(self.rule);
        // index the rule by symbol (always ordered)
        self.rules
            .rule_by_symbols
            .insert(rule_key, rule_ptr.get_index());

        rule_ptr
    }

    pub fn redex(&mut self, ctr: CellPtr, fun: CellPtr) -> EquationPtr {
        let eqn_ptr = self.rules.body.alloc(Equation::redex(ctr, fun));
        self.rule.body.push(eqn_ptr);
        eqn_ptr
    }

    pub fn bind(&mut self, var: PVarPtr, cell: CellPtr) -> EquationPtr {
        let eqn_ptr = self.rules.body.alloc(Equation::bind(var, cell));
        self.rule.body.push(eqn_ptr);
        eqn_ptr
    }

    pub fn connect(&mut self, left: PVarPtr, right: PVarPtr) -> EquationPtr {
        assert!(
            left.get_polarity() == right.get_polarity().flip(),
            "Short-circuit!"
        );
        let eqn_ptr = self.rules.body.alloc(Equation::connect(left, right));
        self.rule.body.push(eqn_ptr);
        eqn_ptr
    }

    /// ------------------------------------------------

    pub fn cell0(&mut self, name: &SymbolName) -> CellPtr {
        let symbol_ptr = self.rules.symbols.get_by_name(name).unwrap();
        self.rules.heap.cell0(symbol_ptr)
    }

    pub fn cell1(&mut self, name: &SymbolName, port: TermPtr) -> CellPtr {
        let symbol_ptr = self.rules.symbols.get_by_name(name).unwrap();
        // check polarity
        assert!(
            port.get_polarity()
                .is_opposite(self.rules.symbols.get(symbol_ptr).get_left_polarity()),
            "Short-circuit connecting port for {:?}",
            symbol_ptr
        );
        self.rules.heap.cell1(symbol_ptr, port)
    }

    pub fn cell2(&mut self, name: &SymbolName, left_port: TermPtr, right_port: TermPtr) -> CellPtr {
        let symbol_ptr = self.rules.symbols.get_by_name(name).unwrap();
        // check left polarity
        assert!(
            left_port
                .get_polarity()
                .is_opposite(self.rules.symbols.get(symbol_ptr).get_left_polarity()),
            "Short-circuit connecting left port for {}",
            self.rules.symbols.display_symbol(symbol_ptr)
        );
        // check right polarity
        assert!(
            right_port
                .get_polarity()
                .is_opposite(self.rules.symbols.get(symbol_ptr).get_right_polarity()),
            "Short-circuit connecting right port for {}",
            self.rules.symbols.display_symbol(symbol_ptr)
        );
        self.rules.heap.cell2(symbol_ptr, left_port, right_port)
    }

    /// ------------------------------------------------
    fn get_port_polarity(&self, port: RulePort) -> Polarity {
        let (symbol_ptr, symbol, port_num) = match port {
            RulePort::Ctr(port_num) => (self.rule.ctr_ptr, self.ctr_symbol, port_num),
            RulePort::Fun(port_num) => (self.rule.fun_ptr, self.fun_symbol, port_num),
        };
        match port_num {
            PortNum::Zero => {
                assert!(
                    symbol.get_arity() >= SymbolArity::One,
                    "Symbol {} has no ports",
                    self.rules.symbols.display_symbol(symbol_ptr)
                );
                return symbol.get_left_polarity();
            }
            PortNum::One => {
                assert!(
                    symbol.get_arity() == SymbolArity::Two,
                    "Symbol {} does not have two ports",
                    self.rules.symbols.display_symbol(symbol_ptr)
                );
                return symbol.get_right_polarity();
            }
        }
    }

    fn port_var(&mut self, port: RulePort) -> PVarPtr {
        let var_ptr = self.rules.heap.fvar(port);
        let (neg_pvar, pos_pvar) = PVarPtr::wire(var_ptr);
        match self.get_port_polarity(port) {
            crate::inet::Polarity::Pos => {
                self.rule.fvar_ptrs.push(pos_pvar);
                return neg_pvar;
            }
            crate::inet::Polarity::Neg => {
                self.rule.fvar_ptrs.push(neg_pvar);
                return pos_pvar;
            }
        }
    }

    pub fn ctr_port_0(&mut self) -> PVarPtr {
        self.port_var(RulePort::Ctr(PortNum::Zero))
    }

    pub fn ctr_port_1(&mut self) -> PVarPtr {
        self.port_var(RulePort::Ctr(PortNum::One))
    }

    pub fn fun_port_0(&mut self) -> PVarPtr {
        self.port_var(RulePort::Fun(PortNum::Zero))
    }

    pub fn fun_port_1(&mut self) -> PVarPtr {
        self.port_var(RulePort::Fun(PortNum::One))
    }

    pub fn var(&mut self) -> (PVarPtr, PVarPtr) {
        self.rule.bvar_count += 1;
        let bvar = self.rules.heap.bvar(self.rule.bvar_count - 1);
        PVarPtr::wire(bvar)
    }
}

#[derive(Debug)]
pub struct RuleSet<'a> {
    symbols: &'a SymbolBook,
    rules: Rules,
    rule_by_symbols: HashMap<RuleKey, usize>,
    pub(crate) body: Equations<RuleF>,
    pub(crate) heap: Heap<RuleF>,
}

impl<'a> RuleSet<'a> {
    fn to_key(left: SymbolPtr, right: SymbolPtr) -> RuleKey {
        (
            std::cmp::min(left.get_index(), right.get_index()),
            std::cmp::max(left.get_index(), right.get_index()),
        )
    }

    pub fn new(symbols: &'a SymbolBook) -> Self {
        Self {
            symbols,
            rules: Rules::new(),
            rule_by_symbols: HashMap::default(),
            body: Equations::new(),
            heap: Heap::new(),
        }
    }

    pub fn rule<F>(&mut self, ctr_name: &SymbolName, fun_name: &SymbolName, body: F) -> RulePtr
    where
        F: FnOnce(&mut RuleBuilder),
    {
        // create the body
        let ctr_ptr = self.symbols.get_by_name(ctr_name).unwrap();
        let fun_ptr = self.symbols.get_by_name(fun_name).unwrap();

        let mut builder = RuleBuilder::new(ctr_ptr, fun_ptr, self);
        body(&mut builder);
        builder.build()
    }

    pub fn get_by_symbols(&self, ctr: SymbolPtr, fun: SymbolPtr) -> Option<RulePtr> {
        let key = RuleSet::to_key(ctr, fun);
        match self.rule_by_symbols.get(&key) {
            Some(index) => Some(RulePtr::new(*index)),
            None => None,
        }
    }

    pub fn get_rule(&'a self, rule_ptr: RulePtr) -> &'a Rule {
        self.rules.get(rule_ptr).unwrap()
    }

    pub fn get_equation(&'a self, ptr: EquationPtr) -> &'a Equation<RuleF> {
        self.body.get(ptr).unwrap()
    }

    pub fn display_rule(&'a self, rule_ptr: RulePtr) -> RuleDisplay {
        RuleDisplay {
            rule_ptr,
            rules: self,
        }
    }

    fn display_body(&'a self, body: &'a [EquationPtr]) -> RuleBodyDisplay {
        RuleBodyDisplay { body, rules: self }
    }

    pub fn display_cell(&'a self, cell_ptr: CellPtr) -> CellDisplay<RuleF> {
        self.heap.display_cell(self.symbols, cell_ptr)
    }

    pub fn display_var(&'a self, var_ptr: VarPtr) -> VarDisplay<RuleF> {
        self.heap.display_var(self.symbols, var_ptr)
    }

    pub fn display_equation(&'a self, equation: &'a Equation<RuleF>) -> EquationDisplay<RuleF> {
        EquationDisplay {
            equation,
            symbols: self.symbols,
            heap: &self.heap,
        }
    }
}

impl<'a> Display for RuleSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.rules.iter().fold(Ok(()), |result, rule_ptr| {
            result.and_then(|_| writeln!(f, "{}", self.display_rule(rule_ptr)))
        })
    }
}

pub struct RuleDisplay<'a> {
    rule_ptr: RulePtr,
    rules: &'a RuleSet<'a>,
}
impl<'a> Display for RuleDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rule = self.rules.get_rule(self.rule_ptr);

        let ctr_name = self.rules.symbols.get_name(rule.ctr_ptr).unwrap();
        let fun_name = self.rules.symbols.get_name(rule.fun_ptr).unwrap();

        match rule.fun_ptr.get_arity() {
            SymbolArity::Zero => write!(f, "{}", fun_name),
            SymbolArity::One => write!(f, "({} F₀)", fun_name),
            SymbolArity::Two => write!(f, "({} F₀ F₁)", fun_name),
        }
        .and_then(|_| write!(f, " ⋈ "))
        .and_then(|_| match rule.ctr_ptr.get_arity() {
            SymbolArity::Zero => write!(f, "{}", ctr_name),
            SymbolArity::One => write!(f, "({} C₀)", ctr_name),
            SymbolArity::Two => write!(f, "({} C₀ C₁)", ctr_name),
        })
        .and_then(|_| write!(f, "  ⟶  "))
        .and_then(|_| self.rules.display_body(&rule.body[..]).fmt(f))
    }
}

pub struct RuleBodyDisplay<'a> {
    pub body: &'a [EquationPtr],
    pub rules: &'a RuleSet<'a>,
}
impl<'a> Display for RuleBodyDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.body
            .iter()
            .fold(Ok(()), |result, eqn_ptr| {
                result.and_then(|_| {
                    let equation = self.rules.body.get(*eqn_ptr).unwrap();
                    write!(f, ", {}", self.rules.display_equation(equation))
                })
            })
            .and_then(|_| write!(f, ""))
    }
}

mod tests {

    use crate::inet::Polarity;

    use super::*;

    #[test]
    fn test_rule_ptrs() {
        let mut ptr = RulePtr::new(42);
        assert_eq!(ptr.get_index(), 42);
        ptr.set_index(43);
        assert_eq!(ptr.get_index(), 43);
    }

    #[test]
    fn test_rule_add_and_get() {
        let mut rules = Rules::new();
        let mut symbols = SymbolBook::new();
        let ctr = symbols.ctr0(&"Ctr".into());
        let fun = symbols.fun0(&"Fun".into());
        let rule = Rule::new(ctr, fun);
        // let ptr = rules.alloc(rule.clone());

        // assert_eq!(rules.get(ptr), &rule);
    }

    #[test]
    fn test_rule_add_all() {
        let mut rules = Rules::new();
        let mut symbols = SymbolBook::new();
        let ctr1 = symbols.ctr0(&"Ctr1".into());
        let fun1 = symbols.fun0(&"Fun1".into());
        let rule1 = Rule::new(ctr1, fun1);
        // let ptr1 = rules.alloc(rule1.clone());

        let ctr2 = symbols.ctr0(&"Ctr2".into());
        let fun2 = symbols.fun0(&"Fun2".into());
        let rule2 = Rule::new(ctr2, fun2);
        // let ptr2 = rules.alloc(rule2.clone());

        let mut all_rules = Rules::new();
        // all_rules.extends(rules);

        // assert_eq!(all_rules.get(ptr1).unwrap(), &rule1);
        // assert_eq!(all_rules.get(ptr2).unwrap(), &rule2);
    }

    // #[test]
    // fn test_rule_item_display() {
    //     let mut rules = Rules::new();
    //     let symbols = SymbolBook::new();
    //     let ctr = symbols.get_or_intern("Ctr");
    //     let fun = symbols.get_or_intern("Fun");
    //     let eqn = Equation::new(vec![], EquationKind::Variable);
    //     let ptr = rules.add(Rule::new(ctr, fun, 0, vec![eqn.to_ptr(0)])).to_ptr(0);

    //     let book = RuleSet::new(symbols, rules, Cells::new(), Equations::new(), FVars::new(), BVars::new());

    //     let rule_item = RuleDisplay {
    //         rule_ptr: ptr,
    //         symbols: &book.symbols,
    //         book: &book,
    //     };
    //     let rule_str = format!("{}", rule_item);
    //     assert_eq!(rule_str, "Ctr ⋈ Fun  ⟶  , ?0 := 0");
    // }
}
