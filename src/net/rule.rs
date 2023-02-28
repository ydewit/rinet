use std::{default, collections::HashMap};

use super::{dsl::{RuleDsl, TermDsl}, arena::{Arena, Ptr, ToTag}, symbol::{SymbolTag}, term::{CellTag, VarTag, Net, Cell}};

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

// #[derive(Debug)]
// pub struct RuleVar {
//     name: char
// }

pub struct Rule {
    ctr: Ptr<SymbolTag>,
    fun: Ptr<SymbolTag>,
    body: RuleBody
}

impl ToTag<RuleTag> for Rule {
    fn to_tag(&self) -> RuleTag {
        RuleTag {}
    }
}


pub type RuleBody = Net<RulePort, char>;
impl RuleBody {
    pub fn new() -> RuleBody {
        Net::default()
    }

    pub fn ctr_port_0(&mut self) -> Ptr<VarTag> {
        self.fvar(RulePort::Ctr { port: PortNum::Zero })
    }

    pub fn ctr_port_1(&mut self) -> Ptr<VarTag> {
        self.fvar(RulePort::Ctr { port: PortNum::One })
    }

    pub fn fun_port_0(&mut self) -> Ptr<VarTag> {
        self.fvar(RulePort::Fun { port: PortNum::One })
    }

    pub fn fun_port_1(&mut self) -> Ptr<VarTag> {
        self.fvar(RulePort::Fun { port: PortNum::One })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RuleTag {}

pub struct RuleBook {
    rule_by_head: HashMap<(Ptr<SymbolTag>, Ptr<SymbolTag>), Ptr<RuleTag>>,
    rules: Arena<Rule, RuleTag>
}

impl RuleBook {
    pub fn get_rule(&self, ptr: &Ptr<RuleTag>) -> Option<&Rule> {
        self.rules.get(ptr)
    }

    pub fn get_rule_by_symbols(&self, ctr_symbol: &Ptr<SymbolTag>, fun_symbol: &Ptr<SymbolTag>) -> Option<&Ptr<RuleTag>> {
        let key = (*ctr_symbol, *fun_symbol);
        self.rule_by_head.get(&key)
    }
}
impl Default for RuleBook {
    fn default() -> Self {
        Self { rules: Arena::new(), rule_by_head: Default::default() }
    }
}

impl RuleDsl for RuleBook {
    type RuleRef = Ptr<RuleTag>;
    type RuleBodyDsl = RuleBody;

    type CtrSymbolRef = Ptr<SymbolTag>;
    type FunSymbolRef = Ptr<SymbolTag>;

    fn rule<F>(&mut self, ctr: Self::CtrSymbolRef, fun: Self::CtrSymbolRef, body_fn: F) -> Self::RuleRef
            where F: FnOnce(&mut Self::RuleBodyDsl){
        let mut body = RuleBody::new();
        body_fn(&mut body);
        let ptr = self.rules.alloc(Rule { ctr, fun, body }, RuleTag {});
        self.rule_by_head.insert((ctr, fun), ptr);
        return ptr
    }
}


struct BindEntry {
    var: Ptr<VarTag>,
    cell: Ptr<CellTag>
}


struct ConnectEntry {
    left: Ptr<VarTag>,
    right: Ptr<VarTag>
}
