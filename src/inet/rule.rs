use core::panic;
use std::{collections::HashMap, fmt::{Display}};

use super::{symbol::{SymbolPtr, SymbolArity, SymbolBook}, equation::{EquationPtr, Equation, Equations, EquationKind, EquationItem}, cell::{CellPtr, PortPtr, Cells, PortKind}, var::{VarPtr, FVarPtr, BVarPtr, BVar, FVar, BVars, FVars, VarKind, VarStore}, arena::{ToPtr, ArenaIter}, BitSet16};

#[derive(Debug)]
pub enum PortNum {
    Zero = 00,
    One = 1
}

#[derive(Debug)]
pub enum RulePort {
    Ctr { port: PortNum },
    Fun { port: PortNum }
}

#[derive(Clone,Copy, PartialEq)]
pub struct RulePtr(u16);
impl RulePtr {
    const INDEX     : BitSet16<14> = BitSet16{ mask: 0b00111111_11111111, offset: 0 };

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

type RuleId = (usize, usize);

#[derive(Debug,Clone,PartialEq)]
pub struct Rule {
    ctr: SymbolPtr,
    fun: SymbolPtr,
    bvars_count: usize,
    pub body: Vec<EquationPtr>
}
impl Rule {
    pub fn new(ctr: SymbolPtr, fun: SymbolPtr) -> Self {
        Self{ ctr, fun, bvars_count: 0, body: Vec::new() }
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


pub struct RulesItem<'a> {
    pub symbols: &'a SymbolBook,
    pub book: &'a RuleBook,
}
impl<'a> RulesItem<'a> {
    fn to_rule_item(&self, rule_ptr: RulePtr) -> RuleItem {
        RuleItem { rule_ptr, symbols: self.symbols, book: self.book }
    }
}
impl<'a> Display for RulesItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.book.rules.iter().fold(Ok(()), |result, rule_ptr| {
            result.and_then(|_| writeln!(f, "{}", self.to_rule_item(rule_ptr)))
        })
    }
}

pub struct RuleItem<'a> {
    rule_ptr: RulePtr,
    symbols: &'a SymbolBook,
    book: &'a RuleBook,
}
impl<'a> RuleItem<'a> {
    fn to_body_item(&self, body: &'a Vec<EquationPtr>) -> RuleBodyItem {
        RuleBodyItem {
            body,
            symbols: self.symbols,
            book: self.book,
        }
    }
}
impl<'a> Display for RuleItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rule = self.book.get_rule(self.rule_ptr);

        let ctr_name = self.symbols.get_name(rule.ctr);
        let fun_name = self.symbols.get_name(rule.fun);
        write!(f, "{} >< {} -->{}", ctr_name, fun_name, self.to_body_item(&rule.body))
    }
}

pub struct RuleBodyItem<'a> {
    pub body: &'a Vec<EquationPtr>,
    pub symbols: &'a SymbolBook,
    pub book: &'a RuleBook,
}
impl<'a> RuleBodyItem<'a> {
    fn to_equation_item(&self, eqn_ptr: EquationPtr) -> EquationItem<'a, RuleFreeStore, RuleBoundStore> {
        EquationItem {
            ptr: eqn_ptr,
            symbols: self.symbols,
            equations: &self.book.equations,
            cells: &self.book.cells,
            bvars: &self.book.bvars,
            fvars: &self.book.fvars,
        }
    }
}
impl<'a> Display for RuleBodyItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.body.iter().fold(Ok(()), |result, eqn_ptr| {
            result.and_then(|_| write!(f, ", {}", self.to_equation_item(*eqn_ptr)))
        }).and_then(|_| write!(f, ""))
    }
}


impl ToPtr<RulePtr> for Rule {
    fn to_ptr(&self, index: usize) -> RulePtr {
        RulePtr::new(index)
    }
}

#[derive(Debug)]
pub struct Rules(Vec<Rule>);
impl Rules {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn get(&self, ptr: RulePtr) -> &Rule {
        &self.0[ptr.get_index()]
    }

    pub fn add_all(&mut self, rules: Rules) {
        self.0.extend(rules.0)
    }

    pub fn add(&mut self, rule: Rule) -> RulePtr {
        let index = self.0.len();
        let rule_ptr = rule.to_ptr(index);
        self.0.push(rule);
        rule_ptr
    }

    pub fn iter(&self) -> ArenaIter<Rule,RulePtr> {
        ArenaIter::new(&self.0)
    }
}

type RuleBoundStore = ();
impl VarStore for RuleBoundStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, var_ptr: VarPtr) -> std::fmt::Result {
        write!(f, "x[{}]", var_ptr.get_index())
    }
}

type RuleFreeStore = RulePort;
impl VarStore for RuleFreeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _: VarPtr) -> std::fmt::Result {
        match self {
            RulePort::Ctr { port: PortNum::Zero } => write!(f, "{{Ctr[0]}}"),
            RulePort::Ctr { port: PortNum::One } => write!(f, "{{Ctr[1]}}"),
            RulePort::Fun { port: PortNum::Zero } => write!(f, "{{fun[0]}}"),
            RulePort::Fun { port: PortNum::One } => write!(f, "{{fun[1]}}")
        }
    }
}

impl BVars<RuleBoundStore> {
    pub fn bvar(&mut self) -> BVarPtr {
        self.add(BVar::new(()))
    }
}

impl FVars<RuleFreeStore> {
    pub fn fvar(&mut self, port: RulePort) -> FVarPtr {
        self.add(FVar::new(port))
    }
}

pub struct RuleBuilder<'a> {
    ctr: SymbolPtr,
    fun: SymbolPtr,
    book: &'a mut RuleBook,
    rule: Rule
}

impl<'a> RuleBuilder<'a> {
    fn new(ctr: SymbolPtr, fun: SymbolPtr, book: &'a mut RuleBook) -> Self {
        Self{
            ctr,
            fun,
            book,
            rule: Rule::new(ctr, fun)
        }
    }

    fn build(&mut self) -> RulePtr {
        let rule_ptr = self.book.rules.add(self.rule.clone());
        self.book.rule_by_symbols.insert((self.ctr.get_index(), self.fun.get_index()), rule_ptr.get_index());
        rule_ptr
    }

    fn build_rule(&mut self) {
        for eqn_ptr in self.book.equations.iter() {
            let eqn = self.book.equations.get(eqn_ptr);
            match eqn.get_kind() {
                EquationKind::Redex => {
                    self.build_cell(eqn.get_redex_ctr());
                    self.build_cell(eqn.get_redex_fun());
                },
                EquationKind::Bind => {
                    self.build_var(eqn.get_bind_var());
                    self.build_cell(eqn.get_bind_cell());
                },
                EquationKind::Connect => {
                    self.build_var(eqn.get_connect_left());
                    self.build_var(eqn.get_connect_right());
                },
            }
        }
    }

    fn build_cell(&self, cell_ptr: CellPtr) {
        let cell = self.book.cells.get(cell_ptr);
        match cell.get_symbol_ptr().get_arity() {
            SymbolArity::Zero => {

            },
            SymbolArity::One => {
                self.build_port(cell.get_left_port());
            },
            SymbolArity::Two => {
                self.build_port(cell.get_left_port());
                self.build_port(cell.get_right_port());
            },
        }
    }

    fn build_port(&self, port_ptr: PortPtr) {
        match port_ptr.get_kind() {
            PortKind::Cell => {
                self.build_cell(port_ptr.get_cell());
            },
            PortKind::Var => {
                self.build_var(port_ptr.get_var());
            },
        }
    }

    fn build_var(&self, var_ptr: VarPtr) {
        match var_ptr.get_kind() {
            VarKind::Bound => {
                let bvar = self.book.bvars.get(var_ptr.into());
            },
            VarKind::Free => {
                let fvar = self.book.fvars.get(var_ptr.into());
            },
        }
    }

    pub fn redex(&mut self, ctr: CellPtr, fun: CellPtr) -> EquationPtr {
        let eqn_ptr = self.book.equations.redex(ctr, fun);
        self.rule.body.push(eqn_ptr);
        eqn_ptr
    }

    pub fn bind(&mut self, var: VarPtr, cell: CellPtr) -> EquationPtr {
        let eqn_ptr = self.book.equations.bind(var, cell);
        self.rule.body.push(eqn_ptr);
        eqn_ptr
    }

    pub fn connect(&mut self, left: VarPtr, right: VarPtr) -> EquationPtr {
        let eqn_ptr = self.book.equations.connect(left, right);
        self.rule.body.push(eqn_ptr);
        eqn_ptr
    }

    /// ------------------------------------------------

    pub fn cell0(&mut self, symbol: SymbolPtr) -> CellPtr {
        self.book.cells.cell0(symbol)
    }

    pub fn cell1(&mut self, symbol: SymbolPtr, port: PortPtr) -> CellPtr {
        self.book.cells.cell1(symbol, port)
    }

    pub fn cell2(&mut self, symbol: SymbolPtr, left_port: PortPtr, right_port: PortPtr) -> CellPtr {
        self.book.cells.cell2(symbol, left_port, right_port)
    }

    /// ------------------------------------------------

    pub fn ctr_port_0(&mut self) -> FVarPtr {
        self.book.fvars.add(FVar::new(RulePort::Ctr { port: PortNum::Zero }))
    }

    pub fn ctr_port_1(&mut self) -> FVarPtr {
        self.book.fvars.add(FVar::new(RulePort::Ctr { port: PortNum::One }))
    }

    pub fn fun_port_0(&mut self) -> FVarPtr {
        self.book.fvars.add(FVar::new(RulePort::Fun { port: PortNum::Zero }))
    }

    pub fn fun_port_1(&mut self) -> FVarPtr {
        self.book.fvars.add(FVar::new(RulePort::Fun { port: PortNum::One }))
    }

    pub fn var(&mut self) -> BVarPtr {
        self.rule.bvars_count += 1;
        self.book.bvars.add(BVar::new(()))
    }

}


#[derive(Debug)]
pub struct RuleBook {
    rules: Rules,
    rule_by_symbols: HashMap<(usize,usize), usize>,
    pub equations : Equations,
    pub cells: Cells,
    pub bvars: BVars<()>,
    pub fvars: FVars<RulePort>
}

impl RuleBook {
    pub fn new() -> Self {
        Self {
            rules: Rules::new(),
            rule_by_symbols: HashMap::default(),
            equations : Equations::new(),
            cells: Cells::new(),
            bvars: BVars::new(),
            fvars: FVars::new()
        }
    }

    pub fn new_rule<F: FnOnce(&mut RuleBuilder)>(&mut self, ctr: SymbolPtr, fun: SymbolPtr, body:F) -> RulePtr {
        // create the body
        let mut builder = RuleBuilder::new(ctr, fun, self);
        body(&mut builder);
        builder.build()
    }

    pub fn get_by_symbols(&self, ctr: SymbolPtr, fun: SymbolPtr) -> RulePtr {
        let key = (ctr.get_index(), fun.get_index());
        match self.rule_by_symbols.get(&key) {
            Some(index) => RulePtr::new(*index),
            None => panic!("Rule not found for: {} >< {}", ctr.get_index(), fun.get_index()),
        }
    }

    pub fn get_rule(&self, rule_ptr: RulePtr) -> &Rule {
        self.rules.get(rule_ptr)
    }

    pub fn get_equation(&self, ptr: EquationPtr) -> &Equation {
        self.equations.get(ptr)
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
        let ptr = rules.add(rule.clone());

        assert_eq!(rules.get(ptr), &rule);
    }

    #[test]
    fn test_rule_add_all() {
        let mut rules = Rules::new();
        let mut symbols = SymbolBook::new();
        let ctr1 = symbols.add_symbol0("Ctr1", Polarity::Pos);
        let fun1 = symbols.add_symbol0("Fun1", Polarity::Neg);
        let rule1 = Rule::new(ctr1, fun1);
        let ptr1 = rules.add(rule1.clone());

        let ctr2 = symbols.add_symbol0("Ctr2", Polarity::Pos);
        let fun2 = symbols.add_symbol0("Fun2", Polarity::Neg);
        let rule2 = Rule::new(ctr2, fun2);
        let ptr2 = rules.add(rule2.clone());

        let mut all_rules = Rules::new();
        all_rules.add_all(rules);

        assert_eq!(all_rules.get(ptr1), &rule1);
        assert_eq!(all_rules.get(ptr2), &rule2);
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

    //     let rule_item = RuleItem {
    //         rule_ptr: ptr,
    //         symbols: &book.symbols,
    //         book: &book,
    //     };
    //     let rule_str = format!("{}", rule_item);
    //     assert_eq!(rule_str, "Ctr >< Fun --> , ?0 := 0");
    // }
}