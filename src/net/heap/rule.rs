use std::{collections::HashMap, sync::{Arc}, cell::Cell};

use super::{symbol::{PortRef, Symbol, SymbolBook}, term::{Var, FVar, Term, Equation, BVar}};
pub use super::term::NetFamily;

#[derive(Debug)]
pub struct RuleFamily {
}

impl NetFamily for RuleFamily {
    type FVar = PortRef;
    type BVar = u8;
}

/*
# Rule
*/
pub type RuleVar = Var<RuleFamily>;
pub type RuleFVar = FVar<RuleFamily>;
pub type RuleCell = Cell<RuleFamily>;
pub type RuleTerm = Term<RuleFamily>;
pub type RuleEquation = Equation<RuleFamily>;

pub type RuleId = (SymbolId, SymbolId);

pub type BVarBuilder = fn(Vec<&RuleFVar>, fn(char) -> BVar) -> Vec<RuleEquation>;

#[derive(Debug)]
struct Rule {
    pub rule_id: RuleId,
    lhs_head: Vec<Arc<RuleFVar>>,
    rhs_head: Vec<Arc<RuleFVar>>,
    pub body: Vec<Box<RuleEquation>>,
    bvars: u8
}

impl Rule {
    fn builder(symbols: &Arc<SymbolBook>, lhs: &Arc<Symbol>, rhs: &Arc<Symbol>) -> RuleBuilder {
        RuleBuilder::new(symbols, lhs, rhs)
    }
}

#[derive(Debug)]
pub struct RuleBuilder {
    rule_id: RuleId,
    symbols: Arc<SymbolBook>,
    lhs_head: Vec<Arc<RuleFVar>>,
    rhs_head: Vec<Arc<RuleFVar>>,
    body: Vec<Box<RuleEquation>>,
    bvars: u8
}

impl RuleBuilder {
    fn new(symbols: &Arc<SymbolBook>, lhs: &Arc<Symbol>, rhs: &Arc<Symbol>) -> Self {
        let rule_id = (lhs.id, rhs.id);
        let mut lhs_head = Vec::with_capacity(lhs.arity());
        for l in 0 .. lhs.arity() {
            lhs_head.push(Arc::new(PortRef { symbol: lhs.clone(), port: l as u8}));
        }
        let mut rhs_head = Vec::with_capacity(rhs.arity());
        for r in 0 .. rhs.arity() {
            rhs_head.push(Arc::new(PortRef { symbol: rhs.clone(), port: r as u8}));
        }
        Self { rule_id, symbols: symbols.clone(), lhs_head, rhs_head, body: Vec::new(), bvars: 0 }
    }


    pub fn fresh(&mut self) -> (RuleVar, RuleVar) {
        let bvar = Arc::new(self.bvars);
        self.bvars += 1;
        (RuleVar::BVar(bvar.clone()), RuleVar::BVar(bvar))
    }

    pub fn lhs_fvar(&self, port: u8) -> RuleVar {
        let fvar = self.lhs_head.get(port as usize).unwrap();
        RuleVar::FVar(fvar.clone())
    }

    pub fn rhs_fvar(&self, port: u8) -> RuleVar {
        let fvar = self.rhs_head.get(port as usize).unwrap();
        RuleVar::FVar(fvar.clone())
    }

    pub fn cell(&self, name: &str, ports: Vec<Box<RuleTerm>>) -> Box<RuleCell> {
        let symbol = self.symbols.find(name).unwrap();
        let cell = Cell::new(symbol, ports);
        Box::new(cell)
    }

    pub fn redex(&mut self, lhs: Box<RuleCell>, rhs: Box<RuleCell>) -> &Self {
        let eqn = RuleEquation::Redex(lhs, rhs);
        self.body.push(Box::new(eqn));
        self
    }

    pub fn connect(&mut self, lhs: RuleVar, rhs: RuleVar) -> &Self {
        let eqn = RuleEquation::Connect(lhs, rhs);
        self.body.push(Box::new(eqn));
        self
    }

    pub fn bind(&mut self, var: RuleVar, cell: Box<RuleCell>) -> &Self {
        let eqn = RuleEquation::Bind(var, cell);
        self.body.push(Box::new(eqn));
        self
    }

    fn build(self) -> Rule {
        // TODO check ref counts in fvars and bvars
        Rule {
            rule_id: self.rule_id,
            lhs_head: self.lhs_head,
            rhs_head: self.rhs_head,
            body: self.body,
            bvars: self.bvars
        }
    }
}

    // /// Creates a new [`Rule`].
    // fn create(lhs: &Symbol, rhs: &Symbol, body: BVarBuilder) -> Rule {
    //     let mut bvars = 0;
    //     let bvar_creator: fn(char) -> Arc<Var<RuleFamily>> = |name| {
    //         bvars = bvars + 1;
    //         Arc::new(Var::BVar(RuleFamily::BVar()))
    //     };

    //     let rule_id: RuleId = (Symbol::to_symbol_id(lhs), Symbol::to_symbol_id(rhs));

    //     let mut head = Vec::with_capacity(lhs.arity() + rhs.arity());
    //     for l in 0..lhs.arity() {
    //         head[l] = Var::PortRef { symbol: lhs, port: l as u8 }
    //     }
    //     for r in 0..rhs.arity() {
    //         head[lhs.arity() + r] = PortRef { symbol: lhs, port: r as u8 }
    //     }
    //     let body = body(head, bvar_creator);
    //     Rule { rule_id, net: Net { head, body, bvars } }


/*
# RuleBook
*/
#[derive(Debug)]
pub struct RuleBook {
    symbols: Arc<SymbolBook>,
    rules: HashMap<(SymbolId, SymbolId), Rule>
}

impl RuleBook {
    pub fn new(symbols: &Arc<SymbolBook>) -> Self {
        Self { symbols: symbols.clone(), rules: HashMap::new() }
    }

    pub fn add_rule(self: &mut Self, lhs: &str, rhs: &str, build_rule: fn(&mut RuleBuilder) -> ()) {
        if let Some(lhs) = self.symbols.find(lhs) {
            if let Some(rhs) = self.symbols.find(rhs) {
                let mut builder = Rule::builder(&self.symbols, lhs, rhs);
                build_rule(&mut builder);
                let rule = builder.build();
                self.rules.insert(rule.rule_id, rule);
            }
        }
    }

    pub fn find(&self, rule_id: &(SymbolId, SymbolId)) -> Option<&Rule> {
        self.rules.get(rule_id)
    }
}
