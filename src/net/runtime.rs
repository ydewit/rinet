use super::{symbol::SymbolBook, rule::{RuleBook, RuleTag}, term::{Net, CellTag, VarTag, Equation}, arena::Ptr};



pub struct Runtime {
    symbols: SymbolBook,
    rules: RuleBook
}

impl Runtime {
    pub fn new(symbols: SymbolBook, rules: RuleBook) -> Self {
        Self {
            symbols,
            rules
        }
    }

    pub fn eval(&self, mut net: Net) {
        let eqns = net.equations();
        for eqn_ptr in eqns {
            match net.get_equation(&eqn_ptr) {
                Some(Equation::Redex { ctr, fun }) => self.eval_redex(&mut net, ctr, fun),
                Some(Equation::Bind { var, cell }) => self.eval_bind(&mut net, var, cell),
                Some(Equation::Connect { left, right }) => self.eval_connect(&mut net, left, right),
                None => todo!(),
            }
        }
    }

    /// A redex
    fn eval_redex(&self, net: &mut Net, ctr_symbol: &Ptr<CellTag>, fun_symbol: &Ptr<CellTag>) {
        let rule = self.rules.get_rule_by_symbols(&ctr_symbol.tag.get_symbol(), &fun_symbol.tag.get_symbol());
        match rule {
            Some(rule) => {
                self.rewrite(&mut net, ctr_symbol, fun_symbol, rule);
            },
            None => (),
        }
    }

    /// Top-level bind
    fn eval_bind(&self, net: &mut Net, var: &Ptr<VarTag>, cell: &Ptr<CellTag>) {
        todo!()
    }

    /// Top-level connect
    fn eval_connect(&self, net: &mut Net, left: &Ptr<VarTag>, right: &Ptr<VarTag>) {
        todo!()
    }

    fn rewrite(&self, net: &mut Net, ctr_ptr: &Ptr<CellTag>, fun_ptr: &Ptr<CellTag>, rule_ptr: &Ptr<RuleTag>) {
        match self.rules.get_rule(rule_ptr) {
            Some(rule) => {
                let ctr = net.get_cell(ctr_ptr).unwrap();
                let fun = net.get_cell(fun_ptr).unwrap();


            },
            None => panic!("Rule not found for {:?} = {:?}", ctr_ptr, fun_ptr),
        }
    }
}
