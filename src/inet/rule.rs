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
    symbol::{SymbolArity, SymbolBook, SymbolPtr},
    term::{TermFamily, TermPtr},
    var::{Var, VarPtr},
    BitSet16,
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
                write!(f, "x{}", store)
            }
            Var::Free(RulePort::Ctr(PortNum::Zero)) => {
                write!(f, "l₀")
            }
            Var::Free(RulePort::Ctr(PortNum::One)) => {
                write!(f, "l₁")
            }
            Var::Free(RulePort::Fun(PortNum::Zero)) => {
                write!(f, "r₀")
            }
            Var::Free(RulePort::Fun(PortNum::One)) => {
                write!(f, "r₁")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PortNum {
    Zero = 0,
    One = 1,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub(crate) ctr: SymbolPtr,
    pub(crate) fun: SymbolPtr,
    fvar_ptrs: Vec<VarPtr>,
    bvar_count: u8,
    pub body: Vec<EquationPtr>,
}
impl Rule {
    pub fn new(ctr: SymbolPtr, fun: SymbolPtr) -> Self {
        Self {
            ctr,
            fun,
            fvar_ptrs: Vec::new(),
            bvar_count: 0,
            body: Vec::new(),
        }
    }

    pub fn get_key(&self) -> RuleKey {
        (self.ctr.get_index(), self.fun.get_index())
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
    rules: &'b mut RuleBook<'a>,
    rule: Rule,
}

impl<'a, 'b> RuleBuilder<'a, 'b> {
    fn new(ctr: SymbolPtr, fun: SymbolPtr, rules: &'b mut RuleBook<'a>) -> Self {
        Self {
            rules,
            rule: Rule::new(ctr, fun),
        }
    }

    fn build(&mut self) -> RulePtr {
        let rule_ptr = self.rules.rules.alloc(self.rule.clone());
        self.rules
            .rule_by_symbols
            .insert(self.rule.get_key(), rule_ptr.get_index());
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
        let var_ptr = self.rules.heap.fvar(RulePort::Ctr(PortNum::Zero));
        self.rule.fvar_ptrs.push(var_ptr);
        var_ptr
    }

    pub fn ctr_port_1(&mut self) -> VarPtr {
        let var_ptr = self.rules.heap.fvar(RulePort::Ctr(PortNum::One));
        self.rule.fvar_ptrs.push(var_ptr);
        var_ptr
    }

    pub fn fun_port_0(&mut self) -> VarPtr {
        let var_ptr = self.rules.heap.fvar(RulePort::Fun(PortNum::Zero));
        self.rule.fvar_ptrs.push(var_ptr);
        var_ptr
    }

    pub fn fun_port_1(&mut self) -> VarPtr {
        let var_ptr = self.rules.heap.fvar(RulePort::Fun(PortNum::One));
        self.rule.fvar_ptrs.push(var_ptr);
        var_ptr
    }

    pub fn var(&mut self) -> VarPtr {
        self.rule.bvar_count += 1;
        self.rules.heap.bvar(self.rule.bvar_count - 1)
    }
}

#[derive(Debug)]
pub struct RuleBook<'a> {
    symbols: &'a SymbolBook,
    rules: Rules,
    rule_by_symbols: HashMap<RuleKey, usize>,
    pub(crate) body: Equations<RuleF>,
    pub(crate) heap: Heap<RuleF>,
}

impl<'a> RuleBook<'a> {
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

    pub fn rule<F>(&mut self, ctr: SymbolPtr, fun: SymbolPtr, body: F) -> RulePtr
    where
        F: FnOnce(&mut RuleBuilder),
    {
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

impl<'a> Display for RuleBook<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.rules.iter().fold(Ok(()), |result, rule_ptr| {
            result.and_then(|_| writeln!(f, "{}", self.display_rule(rule_ptr)))
        })
    }
}

pub struct RuleDisplay<'a> {
    rule_ptr: RulePtr,
    rules: &'a RuleBook<'a>,
}
impl<'a> Display for RuleDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rule = self.rules.get_rule(self.rule_ptr);

        let ctr_name = self.rules.symbols.get_name(rule.ctr).unwrap();
        let fun_name = self.rules.symbols.get_name(rule.fun).unwrap();

        match rule.ctr.get_arity() {
            SymbolArity::Zero => write!(f, "{}", ctr_name),
            SymbolArity::One => write!(f, "({} l₀)", ctr_name),
            SymbolArity::Two => write!(f, "({} l₀ l₁)", ctr_name),
        }
        .and_then(|_| write!(f, " ⋈ "))
        .and_then(|_| match rule.fun.get_arity() {
            SymbolArity::Zero => write!(f, "{}", fun_name),
            SymbolArity::One => write!(f, "({} r₀)", fun_name),
            SymbolArity::Two => write!(f, "({} r₀ r₁)", fun_name),
        })
        .and_then(|_| write!(f, "  ⟶  "))
        .and_then(|_| self.rules.display_body(&rule.body[..]).fmt(f))
    }
}

pub struct RuleBodyDisplay<'a> {
    pub body: &'a [EquationPtr],
    pub rules: &'a RuleBook<'a>,
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
        let ctr = symbols.declare0("Ctr", Polarity::Pos);
        let fun = symbols.declare0("Fun", Polarity::Neg);
        let rule = Rule::new(ctr, fun);
        let ptr = rules.alloc(rule.clone());

        // assert_eq!(rules.get(ptr), &rule);
    }

    #[test]
    fn test_rule_add_all() {
        let mut rules = Rules::new();
        let mut symbols = SymbolBook::new();
        let ctr1 = symbols.declare0("Ctr1", Polarity::Pos);
        let fun1 = symbols.declare0("Fun1", Polarity::Neg);
        let rule1 = Rule::new(ctr1, fun1);
        let ptr1 = rules.alloc(rule1.clone());

        let ctr2 = symbols.declare0("Ctr2", Polarity::Pos);
        let fun2 = symbols.declare0("Fun2", Polarity::Neg);
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
    //     assert_eq!(rule_str, "Ctr >< Fun  ⟶  , ?0 := 0");
    // }
}
