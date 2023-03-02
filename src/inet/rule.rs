use core::panic;
use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
};

use super::{
    arena::{Arena, ArenaPtr, ArenaValue},
    cell::CellPtr,
    equation::{Equation, EquationDisplay, EquationPtr, Equations},
    heap::{CellDisplay, Heap, VarDisplay},
    symbol::{SymbolBook, SymbolPtr},
    term::{TermFamily, TermPtr},
    var::{Var, VarPtr, Vars},
    BitSet16,
};

#[derive(Debug, Clone)]
pub struct RuleF {}
impl TermFamily for RuleF {
    type Store = RuleStore;

    fn display_store(
        f: &mut std::fmt::Formatter<'_>,
        symbols: &SymbolBook,
        heap: &Heap<Self>,
        store: &Self::Store,
        index: usize,
    ) -> std::fmt::Result {
        match store {
            RuleStore::Bound => write!(f, "x[{}]", index),
            RuleStore::Free { port } => match port {
                RulePort::Ctr {
                    port: PortNum::Zero,
                } => write!(f, "{{Ctr[0]}}"),
                RulePort::Ctr { port: PortNum::One } => write!(f, "{{Ctr[1]}}"),
                RulePort::Fun {
                    port: PortNum::Zero,
                } => write!(f, "{{fun[0]}}"),
                RulePort::Fun { port: PortNum::One } => write!(f, "{{fun[1]}}"),
            },
        }
    }
}

#[derive(Debug)]
pub enum RuleStore {
    Bound,
    Free { port: RulePort },
}

impl Vars<RuleF> {
    pub fn bvar(&mut self) -> VarPtr {
        self.alloc(Var::new(RuleStore::Bound))
    }

    pub fn fvar(&mut self, port: RulePort) -> VarPtr {
        self.alloc(Var::new(RuleStore::Free { port }))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PortNum {
    Zero = 0,
    One = 1,
}

#[derive(Debug, Clone, Copy)]
pub enum RulePort {
    Ctr { port: PortNum },
    Fun { port: PortNum },
}

impl Into<RuleStore> for RulePort {
    fn into(self) -> RuleStore {
        RuleStore::Free { port: self }
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

type RuleId = (usize, usize);

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    ctr: SymbolPtr,
    fun: SymbolPtr,
    fvar_ptrs: Vec<VarPtr>,
    bvars_count: usize,
    pub body: Vec<EquationPtr>,
}
impl Rule {
    pub fn new(ctr: SymbolPtr, fun: SymbolPtr) -> Self {
        Self {
            ctr,
            fun,
            fvar_ptrs: Vec::new(),
            bvars_count: 0,
            body: Vec::new(),
        }
    }

    pub fn get_id(&self) -> RuleId {
        (self.ctr.get_index(), self.fun.get_index())
    }

    pub fn get_bvar_count(&self) -> usize {
        self.bvars_count
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

pub struct RuleBuilder<'a> {
    ctr: SymbolPtr,
    fun: SymbolPtr,
    rules: &'a mut RuleBook,
    rule: Rule,
}

impl<'a> RuleBuilder<'a> {
    fn new(ctr: SymbolPtr, fun: SymbolPtr, rules: &'a mut RuleBook) -> Self {
        Self {
            ctr,
            fun,
            rules,
            rule: Rule::new(ctr, fun),
        }
    }

    fn build(&mut self) -> RulePtr {
        let rule_ptr = self.rules.rules.alloc(self.rule.clone());
        self.rules
            .rule_by_symbols
            .insert(RuleBook::to_key(self.ctr, self.fun), rule_ptr.get_index());
        rule_ptr
    }

    pub fn redex(&mut self, ctr: CellPtr, fun: CellPtr) -> EquationPtr {
        let eqn_ptr = self.rules.body.alloc(Equation::redex(ctr, fun));
        self.rule.body.push(eqn_ptr);
        eqn_ptr
    }

    pub fn bind(&mut self, var: VarPtr, cell: CellPtr) -> EquationPtr {
        let eqn_ptr = self.rules.body.alloc(Equation::bind(var, cell));
        self.rule.body.push(eqn_ptr);
        eqn_ptr
    }

    pub fn connect(&mut self, left: VarPtr, right: VarPtr) -> EquationPtr {
        let eqn_ptr = self.rules.body.alloc(Equation::connect(left, right));
        self.rule.body.push(eqn_ptr);
        eqn_ptr
    }

    /// ------------------------------------------------

    pub fn cell0(&mut self, symbol_ptr: SymbolPtr) -> CellPtr {
        self.rules.heap.cell0(symbol_ptr)
    }

    pub fn cell1(&mut self, symbol_ptr: SymbolPtr, port: TermPtr) -> CellPtr {
        self.rules.heap.cell1(symbol_ptr, port)
    }

    pub fn cell2(
        &mut self,
        symbol_ptr: SymbolPtr,
        left_port: TermPtr,
        right_port: TermPtr,
    ) -> CellPtr {
        self.rules.heap.cell2(symbol_ptr, left_port, right_port)
    }

    /// ------------------------------------------------

    pub fn ctr_port_0(&mut self) -> VarPtr {
        self.rules.heap.var(
            RulePort::Ctr {
                port: PortNum::Zero,
            }
            .into(),
        )
    }

    pub fn ctr_port_1(&mut self) -> VarPtr {
        self.rules
            .heap
            .var(RulePort::Ctr { port: PortNum::One }.into())
    }

    pub fn fun_port_0(&mut self) -> VarPtr {
        self.rules.heap.var(
            RulePort::Fun {
                port: PortNum::Zero,
            }
            .into(),
        )
    }

    pub fn fun_port_1(&mut self) -> VarPtr {
        self.rules
            .heap
            .var(RulePort::Fun { port: PortNum::One }.into())
    }

    pub fn var(&mut self) -> VarPtr {
        self.rule.bvars_count += 1;
        self.rules.heap.var(RuleStore::Bound)
    }
}

#[derive(Debug)]
pub struct RuleBook {
    rules: Rules,
    rule_by_symbols: HashMap<(usize, usize), usize>,
    pub body: Equations<RuleF>,
    pub heap: Heap<RuleF>,
}

impl RuleBook {
    fn to_key(left: SymbolPtr, right: SymbolPtr) -> (usize, usize) {
        (
            std::cmp::min(left.get_index(), right.get_index()),
            std::cmp::max(left.get_index(), right.get_index()),
        )
    }

    pub fn new() -> Self {
        Self {
            rules: Rules::new(),
            rule_by_symbols: HashMap::default(),
            body: Equations::new(),
            heap: Heap::new(),
        }
    }

    pub fn new_rule<F: FnOnce(&mut RuleBuilder)>(
        &mut self,
        ctr: SymbolPtr,
        fun: SymbolPtr,
        body: F,
    ) -> RulePtr {
        // create the body
        let mut builder = RuleBuilder::new(ctr, fun, self);
        body(&mut builder);
        builder.build()
    }

    pub fn get_by_symbols(&self, ctr: SymbolPtr, fun: SymbolPtr) -> RulePtr {
        let key = RuleBook::to_key(ctr, fun);
        match self.rule_by_symbols.get(&key) {
            Some(index) => RulePtr::new(*index),
            None => panic!("Rule not found for: {:?} >< {:?}", ctr, fun),
        }
    }

    pub fn get_rule<'a>(&'a self, rule_ptr: RulePtr) -> &'a Rule {
        self.rules.get(rule_ptr).unwrap()
    }

    pub fn get_equation<'a>(&'a self, ptr: EquationPtr) -> &'a Equation<RuleF> {
        self.body.get(ptr).unwrap()
    }

    pub fn display_rules<'a>(&'a self, symbols: &'a SymbolBook) -> RulesDisplay {
        RulesDisplay{
            symbols,
            rules: self,
        }
    }

    pub fn display_rule<'a>(&'a self, symbols: &'a SymbolBook, rule_ptr: RulePtr) -> RuleDisplay {
        RuleDisplay::new(rule_ptr, symbols, self)
    }

    pub fn display_cell<'a>(
        &'a self,
        symbols: &'a SymbolBook,
        cell_ptr: CellPtr,
    ) -> CellDisplay<RuleF> {
        self.heap.display_cell(symbols, cell_ptr)
    }

    pub fn display_var<'a>(
        &'a self,
        symbols: &'a SymbolBook,
        var_ptr: VarPtr,
    ) -> VarDisplay<RuleF> {
        self.heap.display_var(symbols, var_ptr)
    }

    pub fn display_equation<'a>(
        &'a self,
        symbols: &'a SymbolBook,
        eqn_ptr: EquationPtr,
    ) -> EquationDisplay<RuleF> {
        EquationDisplay {
            ptr: eqn_ptr,
            symbols,
            body: &self.body,
            heap: &self.heap,
        }
    }
}

pub struct RulesDisplay<'a> {
    symbols: &'a SymbolBook,
    rules: &'a RuleBook,
}
impl<'a> RulesDisplay<'a> {
    fn to_rule_item(&self, rule_ptr: RulePtr) -> RuleDisplay {
        RuleDisplay {
            rule_ptr,
            symbols: self.symbols,
            rules: self.rules,
        }
    }
}
impl<'a> Display for RulesDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.rules.rules.iter().fold(Ok(()), |result, rule_ptr| {
            result.and_then(|_| writeln!(f, "{}", self.to_rule_item(rule_ptr)))
        })
    }
}

pub struct RuleDisplay<'a> {
    rule_ptr: RulePtr,
    symbols: &'a SymbolBook,
    rules: &'a RuleBook,
}
impl<'a> RuleDisplay<'a> {
    pub fn new(rule_ptr: RulePtr, symbols: &'a SymbolBook, rules: &'a RuleBook) -> Self {
        RuleDisplay {
            rule_ptr,
            symbols,
            rules,
        }
    }

    fn to_body_item(&self, body: &'a Vec<EquationPtr>) -> RuleBodyItem {
        RuleBodyItem {
            body,
            symbols: self.symbols,
            rules: self.rules,
        }
    }
}
impl<'a> Display for RuleDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rule = self.rules.get_rule(self.rule_ptr);

        let ctr_name = self.symbols.get_name(rule.ctr);
        let fun_name = self.symbols.get_name(rule.fun);
        write!(
            f,
            "{} >< {} -->{}",
            ctr_name,
            fun_name,
            self.to_body_item(&rule.body)
        )
    }
}

pub struct RuleBodyItem<'a> {
    pub body: &'a Vec<EquationPtr>,
    pub symbols: &'a SymbolBook,
    pub rules: &'a RuleBook,
}
impl<'a> Display for RuleBodyItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.body
            .iter()
            .fold(Ok(()), |result, eqn_ptr| {
                result.and_then(|_| {
                    write!(
                        f,
                        ", {}",
                        self.rules.display_equation(self.symbols, *eqn_ptr)
                    )
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
        let ctr = symbols.add_symbol0("Ctr", Polarity::Pos);
        let fun = symbols.add_symbol0("Fun", Polarity::Neg);
        let rule = Rule::new(ctr, fun);
        let ptr = rules.alloc(rule.clone());

        // assert_eq!(rules.get(ptr), &rule);
    }

    #[test]
    fn test_rule_add_all() {
        let mut rules = Rules::new();
        let mut symbols = SymbolBook::new();
        let ctr1 = symbols.add_symbol0("Ctr1", Polarity::Pos);
        let fun1 = symbols.add_symbol0("Fun1", Polarity::Neg);
        let rule1 = Rule::new(ctr1, fun1);
        let ptr1 = rules.alloc(rule1.clone());

        let ctr2 = symbols.add_symbol0("Ctr2", Polarity::Pos);
        let fun2 = symbols.add_symbol0("Fun2", Polarity::Neg);
        let rule2 = Rule::new(ctr2, fun2);
        let ptr2 = rules.alloc(rule2.clone());

        let mut all_rules = Rules::new();
        // all_rules.extends(rules);

        assert_eq!(all_rules.get(ptr1).unwrap(), &rule1);
        assert_eq!(all_rules.get(ptr2).unwrap(), &rule2);
    }

    // #[test]
    // fn test_rule_item_display() {
    //     let mut rules = Rules::new();
    //     let symbols = SymbolBook::new();
    //     let ctr = symbols.get_or_intern("Ctr");
    //     let fun = symbols.get_or_intern("Fun");
    //     let eqn = Equation::new(vec![], EquationKind::Variable);
    //     let ptr = rules.add(Rule::new(ctr, fun, 0, vec![eqn.to_ptr(0)])).to_ptr(0);

    //     let book = RuleBook::new(symbols, rules, Cells::new(), Equations::new(), FVars::new(), BVars::new());

    //     let rule_item = RuleDisplay {
    //         rule_ptr: ptr,
    //         symbols: &book.symbols,
    //         book: &book,
    //     };
    //     let rule_str = format!("{}", rule_item);
    //     assert_eq!(rule_str, "Ctr >< Fun --> , ?0 := 0");
    // }
}
