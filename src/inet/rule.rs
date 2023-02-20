use std::collections::HashMap;

use super::{symbol::{SymbolPtr}, equation::{EquationPtr, Equation, Equations}, cell::{CellPtr, PortPtr, Cell, Cells}, var::{VarPtr, FVarPtr, BVarPtr, BVar, FVar, BVars, FVars}, arena::{ToPtr, ArenaIter}, BitSet32, BitSet16};

#[derive(Debug)]
pub enum PortNum {
    Zero,
    One
}

#[derive(Debug)]
pub enum RulePort {
    Ctr { port: PortNum },
    Fun { port: PortNum }
}

pub struct RulePtr(u16);
impl RulePtr {
    const INDEX     : BitSet16 = BitSet16{ mask: 0b00111111_11111111, offset: 0 };

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

#[derive(Debug)]
pub struct Rule {
    ctr: SymbolPtr,
    fun: SymbolPtr,
    body: Vec<EquationPtr>
}
impl Rule {
    pub fn new(ctr: SymbolPtr, fun: SymbolPtr, body: Vec<EquationPtr>) -> Self {
        Self{ ctr, fun, body }
    }

    pub fn get_id(&self) -> RuleId {
        (self.ctr.get_index(), self.fun.get_index())
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


pub struct RuleBuilder {
    ctr: SymbolPtr,
    fun: SymbolPtr,
    equations : Equations,
    cells: Cells,
    bvars: BVars<()>,
    fvars: FVars<RulePort>
}

impl RuleBuilder {
    fn new(ctr: SymbolPtr, fun: SymbolPtr) -> Self {
        Self{
            ctr,
            fun,
            equations : Equations::new(),
            cells: Cells::new(),
            bvars: BVars::new(),
            fvars: FVars::new()
        }
    }

    fn build(self, book: &mut RuleBook) -> RulePtr {
        let eqns = self.equations.iter().collect();

        // add the rule
        let rule = Rule::new(self.ctr, self.fun, eqns);

        let rule_ptr = book.rules.add(rule);
        book.rule_by_symbols.insert((self.ctr.get_index(), self.fun.get_index()), rule_ptr.get_index());

        // copy terms
        book.equations.add_all(self.equations);
        book.cells.add_all(self.cells);
        book.bvars.add_all(self.bvars);
        book.fvars.add_all(self.fvars);

        rule_ptr
    }

    // Equations --------------------------

    pub fn redex(&mut self, ctr: CellPtr, fun: CellPtr) -> EquationPtr {
        self.equations.add(Equation::redex(ctr, fun))
    }

    pub fn bind(&mut self, var: VarPtr, cell: CellPtr) -> EquationPtr {
        self.equations.add(Equation::bind(var, cell))
    }

    pub fn connect(&mut self, left: VarPtr, right: VarPtr) -> EquationPtr {
        self.equations.add(Equation::connect(left, right))
    }

    /// ------------------------------------------------

    pub fn cell0(&mut self, symbol: SymbolPtr) -> CellPtr {
        self.cells.add(Cell::new0(symbol))
    }

    pub fn cell1(&mut self, symbol: SymbolPtr, left_port: PortPtr) -> CellPtr {
        self.cells.add(Cell::new1(symbol, left_port))
    }

    pub fn cell2(&mut self, symbol: SymbolPtr, left_port: PortPtr, right_port: PortPtr) -> CellPtr {
        self.cells.add(Cell::new2(symbol, left_port, right_port))
    }

    /// ------------------------------------------------

    pub fn ctr_port_0(&mut self) -> FVarPtr {
        self.fvars.add(FVar::new(RulePort::Ctr { port: PortNum::Zero }))
    }

    pub fn ctr_port_1(&mut self) -> FVarPtr {
        self.fvars.add(FVar::new(RulePort::Ctr { port: PortNum::One }))
    }

    pub fn fun_port_0(&mut self) -> FVarPtr {
        self.fvars.add(FVar::new(RulePort::Fun { port: PortNum::One }))
    }

    pub fn fun_port_1(&mut self) -> FVarPtr {
        self.fvars.add(FVar::new(RulePort::Fun { port: PortNum::One }))
    }

    pub fn var(&mut self) -> BVarPtr {
        self.bvars.add(BVar::new(()))
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
        let mut builder = RuleBuilder::new(ctr, fun);
        body(&mut builder);
        builder.build(self)
    }
}
