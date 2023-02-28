use std::{sync::{Arc}};

use tokio::{spawn};


use super::{rule::{RuleBook, RuleEquation, RuleTerm, RuleCell}, term::{Cell, Term, Var, Net, Equation}, symbol::PortRef};

/*
Net VM
*/


struct NetRuntime {
    book: Arc<RuleBook>
}

impl NetRuntime {
    pub fn eval(&self, net: Net) {
        for eqn in net.body {
            match eqn {
                Equation::Redex(lhs, rhs) => self.eval_redex(lhs, rhs),
                Equation::Bind(var, cell) => self.eval_bind(var, cell),
                Equation::Connect(lhs, rhs) => self.eval_connect(lhs, rhs)
            }
        }
    }

    fn eval_redex(&self, lhs: Box<Cell>, rhs: Box<Cell>) {
        spawn(self.reduce_redex(lhs, rhs));

    }

    fn eval_bind(&self, var: Var, cell: Box<Cell>) {
        match var  {
            Var::FVar(value) => {
                todo!()
            },
            Var::BVar(value) => {
                let other = *value.get_or_init(||{ cell });
                if other.polarity() != cell.polarity() {
                    self.eval_redex(value.take().unwrap(), cell)
                }
            },
        }
    }

    fn eval_connect(&self, lhs: Var, rhs: Var) {
        todo!()
    }

    async fn reduce_redex(&self, lhs: Box<Cell>, rhs: Box<Cell>) {
        let rule_id = (lhs.symbol.id, rhs.symbol.id);
        let Some(rule) = self.book.find(&rule_id) else {
            panic!("rule not found for {:?}", rule_id)
        };

        // let mut bvars = [Option::None;rule.bvar_count];
        for eqn in &rule.body {
            assert!(!eqn.is_short_circuit(), "short circuit: {:?}", eqn);
            self.rewrite_equation(&mut lhs, &mut rhs, eqn);
        }
    }

    fn rewrite_equation(&self, lhs: &mut Box<Cell>, rhs: &mut Box<Cell>, rule_eqn: &RuleEquation) {
        // assert!(!rule_eqn.is_short_circuit(), "short circuit: {:?}", eqn);
        match rule_eqn {
            Equation::Redex(lcell, rcell) => {
                let left = self.instantiate_cell(&lhs, &rhs, lcell);
                let right = self.instantiate_cell(&lhs, &rhs, rcell);
                self.eval_redex(left, right);
            },
            Equation::Bind(rule_var, rule_cell) => {
                let cell = self.instantiate_cell(&lhs, &rhs, rule_cell);
                match rule_var {
                    Var::FVar(port_ref) => {
                        let term = self.resolve_fvar(&mut lhs, &mut rhs, port_ref);
                        match *term {
                            Term::Agent(other) => self.eval_redex(other, cell),
                            Term::Var(var) => self.eval_bind(var, cell),
                        }
                    },
                    Var::BVar(fresh) => self.eval_bind(Var::fresh(), cell)
                }
            },
            Equation::Connect(lhs, rhs) => {
                todo!()
            },
        }
    }

    fn instantiate_term(&self, lhs: &mut Box<Cell>, rhs: &mut Box<Cell>, rule_term: &Box<RuleTerm>) -> Box<Term> {
        match rule_term.as_ref() {
            Term::Agent(cell) => {
                let ports = self.instantiate_ports(&mut lhs, &mut rhs, &cell.ports);
                let cell = Box::new(Cell {symbol: cell.symbol.clone(), ports});
                Box::new(Term::Agent(cell))
            },
            Term::Var(Var::FVar(rule_var)) => {
                self.resolve_fvar(&mut rhs, rule_var)
            },
            Term::Var(Var::BVar(fresh)) => {
                let var = Var::fresh();
                let term = Term::Var(var);
                Box::new(term)
            }
        }
    }

    // fn instantiate_bvar(&self, _: u8) -> Var {
    //     Var::BVar(OnceLock::new())
    // }

    fn resolve_fvar<'a>(&self, lhs: &'a mut Box<Cell>, rhs: &'a mut Box<Cell>, port_ref: &Arc<PortRef>) -> Box<Term> {
        if lhs.is(&port_ref.symbol) {
            lhs.ports[port_ref.port as usize]
        }
        else if rhs.is(&port_ref.symbol) {
            rhs.ports[port_ref.port as usize]
        }
        else {
            panic!("dangling RuleVar {:?}", port_ref);
        }
    }


    fn instantiate_cell(&self, lhs: &Box<Cell>, rhs: &Box<Cell>, rule_cell: &Box<RuleCell>) -> Box<Cell> {
        let ports = self.instantiate_ports(lhs, rhs, &rule_cell.ports);
        Box::new(Cell {symbol: rule_cell.symbol.clone(), ports})
    }

    fn instantiate_ports(&self, lhs: &Box<Cell>, rhs: &Box<Cell>, rule_cell_ports: &Vec<Box<RuleTerm>>) -> Vec<Box<Term>> {
        rule_cell_ports.iter().map(|rule_term| self.instantiate_term(&mut lhs, &mut rhs, rule_term)).collect()
    }
}
