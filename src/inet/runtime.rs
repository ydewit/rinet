use crate::inet::{net::NetStore, var::Var};

use super::{
    cell::{Cell, CellPtr, PortKind, PortPtr},
    equation::{Equation, EquationKind},
    net::{Net, NetF, NetItem},
    rule::{PortNum, RuleBook, RuleF, RuleItem, RulePort, RulePtr, RuleStore},
    symbol::{SymbolArity, SymbolBook},
    var::VarPtr,
    Polarity,
};

pub struct BVarPtrs {
    bvar_ptrs: [VarPtr; Self::MAX_BVARS_PER_RULE],
    len: usize,
}

impl BVarPtrs {
    const MAX_BVARS_PER_RULE: usize = 10;

    fn new(net: &mut Net, len: usize) -> Self {
        assert!(len < Self::MAX_BVARS_PER_RULE);
        let mut this = Self {
            bvar_ptrs: [VarPtr::new(0); Self::MAX_BVARS_PER_RULE],
            len,
        };
        for i in 0..len {
            this.bvar_ptrs[i] = net.vars.add(Var::new(NetStore::default()));
        }
        this
    }

    fn get(&self, bvar_ptr: VarPtr) -> VarPtr {
        self.bvar_ptrs[bvar_ptr.get_index()]
    }
}

pub struct Runtime<'a> {
    symbols: &'a SymbolBook,
    rules: &'a RuleBook,
}

impl<'a> Runtime<'a> {
    pub fn new(symbols: &'a SymbolBook, rules: &'a RuleBook) -> Self {
        Self { symbols, rules }
    }

    pub fn eval(&mut self, mut net: Net) -> Net {
        let eqns: Vec<Equation<NetF>> = net.equations.all().collect();
        for eqn in eqns {
            net = self.eval_equation(net, eqn)
        }
        // let eqn_ptrs: Vec<EquationPtr> = eqns.iter().collect(); // TODO no need for EquationPtr, just use a simple Vec<Equation>
        // for eqn_ptr in eqn_ptrs {
        //     let eqn = eqns.get(eqn_ptr);
        // }

        println!("{}", self.display_net(&net));

        net
        // loop {
        //     if self.deferred.len() == 0 {
        //         break;
        //     }

        //     for eqn in self.deferred.drain(..) {
        //         self.eval_equation(&mut net, eqn)
        //     }
        // }
    }

    fn eval_equation(&mut self, net: Net, eqn: Equation<NetF>) -> Net {
        match eqn.get_kind() {
            EquationKind::Redex => self.eval_redex(net, eqn.get_redex_ctr(), eqn.get_redex_fun()),
            EquationKind::Bind => self.eval_bind(net, eqn.get_bind_var(), eqn.get_bind_cell()),
            EquationKind::Connect => {
                self.eval_connect(net, eqn.get_connect_left(), eqn.get_connect_right())
            }
        }
    }

    fn eval_redex(&mut self, mut net: Net, ctr_ptr: CellPtr, fun_ptr: CellPtr) -> Net {
        println!(
            "Evaluating REDEX: {} >< {}",
            net.display_cell(self.symbols, ctr_ptr),
            net.display_cell(self.symbols, fun_ptr)
        );

        let ctr = net.get_cell(ctr_ptr);
        let fun = net.get_cell(fun_ptr);

        // self.rewrite(net, &ctr, &fun)
        // }

        // pub fn rewrite(&mut self, net: Net, ctr: &Cell<NetF>, fun: &Cell<NetF>) -> Net {
        let rule_ptr = self
            .rules
            .get_by_symbols(ctr.get_symbol_ptr(), fun.get_symbol_ptr());
        let rule = self.rules.get_rule(rule_ptr);

        println!("Found RULE: {}", self.display_rule(rule_ptr));

        // preallocate bound vars (TODO can we allocate in consecutive indexes to simplify rewrite?)
        // let bvars = net.alloc_bvars(rule.get_bvar_count());
        let mut bvars = BVarPtrs::new(&mut net, rule.get_bvar_count());

        // interpret rule
        for rule_eqn_ptr in rule.body() {
            let rule_eqn = self.rules.get_equation(*rule_eqn_ptr);
            net = self.rewrite_rule_equation(net, &mut bvars, ctr, fun, rule_eqn)
        }
        net
    }

    fn rewrite_rule_equation(
        &mut self,
        net: Net,
        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_eqn: Equation<RuleF>,
    ) -> Net {
        match rule_eqn.get_kind() {
            EquationKind::Redex => self.rewrite_redex(
                net,
                bvars,
                ctr,
                fun,
                rule_eqn.get_redex_ctr(),
                rule_eqn.get_redex_fun(),
            ),
            EquationKind::Bind => self.rewrite_bind(
                net,
                bvars,
                ctr,
                fun,
                rule_eqn.get_bind_var(),
                rule_eqn.get_bind_cell(),
            ),
            EquationKind::Connect => self.rewrite_connect(
                net,
                bvars,
                ctr,
                fun,
                rule_eqn.get_connect_left(),
                rule_eqn.get_connect_right(),
            ),
        }
    }

    fn rewrite_redex(
        &mut self,
        mut net: Net,
        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_ctr_ptr: CellPtr,
        rule_fun_ptr: CellPtr,
    ) -> Net {
        let ctr_ptr = self.instantiate_cell(&mut net, bvars, ctr, fun, rule_ctr_ptr);
        let fun_ptr = self.instantiate_cell(&mut net, bvars, ctr, fun, rule_fun_ptr);

        //
        self.eval_redex(net, ctr_ptr, fun_ptr)
    }

    fn rewrite_bind(
        &mut self,
        mut net: Net,
        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_var_ptr: VarPtr,
        rule_cell_ptr: CellPtr,
    ) -> Net {
        let cell_ptr = self.instantiate_cell(&mut net, bvars, ctr, fun, rule_cell_ptr);
        let port_ptr = self.instantiate_var(&net, bvars, ctr, fun, rule_var_ptr);
        match port_ptr.get_kind() {
            PortKind::Cell => match cell_ptr.get_polarity() {
                Polarity::Pos => self.eval_redex(net, cell_ptr, port_ptr.get_cell()),
                Polarity::Neg => self.eval_redex(net, port_ptr.get_cell(), cell_ptr),
            },
            PortKind::Var => {
                match net.vars.try_set(port_ptr.get_var(), cell_ptr) {
                    Some(other_cell_ptr) => match cell_ptr.get_polarity() {
                        Polarity::Pos => self.eval_redex(net, cell_ptr, other_cell_ptr),
                        Polarity::Neg => self.eval_redex(net, other_cell_ptr, cell_ptr),
                    },
                    None => net, // first try_set() call
                }
            }
        }
    }

    fn rewrite_connect(
        &mut self,
        mut net: Net,
        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_left_var: VarPtr,
        rule_right_var: VarPtr,
    ) -> Net {
        let left_port_ptr = self.instantiate_var(&net, bvars, ctr, fun, rule_left_var);
        let right_port_ptr = self.instantiate_var(&net, bvars, ctr, fun, rule_right_var);

        match (left_port_ptr.get_kind(), right_port_ptr.get_kind()) {
            (PortKind::Cell, PortKind::Cell) => match left_port_ptr.get_cell().get_polarity() {
                Polarity::Pos => {
                    self.eval_redex(net, left_port_ptr.get_cell(), right_port_ptr.get_cell())
                }
                Polarity::Neg => {
                    self.eval_redex(net, right_port_ptr.get_cell(), left_port_ptr.get_cell())
                }
            },
            (PortKind::Cell, PortKind::Var) => {
                // bind
                net.equations
                    .bind(right_port_ptr.get_var(), left_port_ptr.get_cell());
                net
            }
            (PortKind::Var, PortKind::Cell) => {
                // bind
                net.equations
                    .bind(left_port_ptr.get_var(), right_port_ptr.get_cell());
                net
            }
            (PortKind::Var, PortKind::Var) => {
                // bind
                net.equations
                    .connect(left_port_ptr.get_var(), right_port_ptr.get_var());
                net
            }
        }
    }

    pub fn instantiate_cell(
        &mut self,
        net: &mut Net,
        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_cell_ptr: CellPtr,
    ) -> CellPtr {
        let rule_cell = self.rules.cells.get(rule_cell_ptr);
        let cell_ptr = match rule_cell.get_symbol_ptr().get_arity() {
            SymbolArity::Zero => net.cells.cell0(rule_cell.get_symbol_ptr()),
            SymbolArity::One => {
                let port_ptr =
                    self.instantiate_port(net, bvars, ctr, fun, rule_cell.get_left_port());
                net.cells.cell1(rule_cell.get_symbol_ptr(), port_ptr)
            }
            SymbolArity::Two => {
                let left_port_ptr =
                    self.instantiate_port(net, bvars, ctr, fun, rule_cell.get_left_port());
                let right_port_ptr =
                    self.instantiate_port(net, bvars, ctr, fun, rule_cell.get_right_port());

                net.cells
                    .cell2(rule_cell.get_symbol_ptr(), left_port_ptr, right_port_ptr)
            }
        };
        println!(
            "Instantiating cell: {} --> {}",
            self.rules.display_cell(self.symbols, rule_cell_ptr),
            net.display_cell(self.symbols, cell_ptr)
        );
        cell_ptr
    }

    fn instantiate_port(
        &mut self,
        mut net: &mut Net,
        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_port_ptr: PortPtr,
    ) -> PortPtr {
        match rule_port_ptr.get_kind() {
            PortKind::Cell => self
                .instantiate_cell(&mut net, bvars, ctr, fun, rule_port_ptr.get_cell())
                .into(),
            PortKind::Var => self.instantiate_var(net, bvars, ctr, fun, rule_port_ptr.get_var()),
        }
    }

    fn instantiate_var(
        &mut self,
        net: &Net,
        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_var_ptr: VarPtr,
    ) -> PortPtr {
        print!(
            "Instantiating var: {}",
            self.rules.display_var(self.symbols, rule_var_ptr)
        );
        let rule_var = self.rules.vars.get(rule_var_ptr);
        let port_ptr = match &rule_var.0 {
            RuleStore::Bound => bvars.get(rule_var_ptr).into(),
            RuleStore::Free { port } => self.resolve_fvar(ctr, fun, *port),
        };
        println!(" --> {}", net.display_port(self.symbols, port_ptr));
        port_ptr
    }

    fn resolve_fvar(&self, ctr: Cell<NetF>, fun: Cell<NetF>, port: RulePort) -> PortPtr {
        match port {
            RulePort::Ctr {
                port: PortNum::Zero,
            } => ctr.get_left_port(),
            RulePort::Ctr { port: PortNum::One } => ctr.get_right_port(),
            RulePort::Fun {
                port: PortNum::Zero,
            } => fun.get_left_port(),
            RulePort::Fun { port: PortNum::One } => fun.get_right_port(),
        }
    }

    fn eval_bind(&mut self, mut net: Net, var_ptr: VarPtr, cell_ptr: CellPtr) -> Net {
        println!(
            "Evaluating BIND: {} <- {}",
            net.display_var(self.symbols, var_ptr),
            net.display_cell(self.symbols, cell_ptr)
        );

        match net.vars.try_set(var_ptr, cell_ptr) {
            Some(other_cell_ptr) => match cell_ptr.get_polarity() {
                Polarity::Pos => self.eval_redex(net, cell_ptr, other_cell_ptr),
                Polarity::Neg => self.eval_redex(net, other_cell_ptr, cell_ptr),
            },
            None => net, // set succeeded,
        }
    }

    fn eval_connect(&mut self, net: Net, left_var_ptr: VarPtr, right_var_ptr: VarPtr) -> Net {
        println!(
            "Evaluating CONNECT: {} <-> {}",
            net.display_var(self.symbols, left_var_ptr),
            net.display_var(self.symbols, right_var_ptr)
        );

        let left_var = net.vars.get(left_var_ptr);
        let right_var = net.vars.get(right_var_ptr);

        todo!()
    }

    pub fn display_net(&self, net: &'a Net) -> NetItem {
        NetItem::new(&self.symbols, &net)
    }

    pub fn display_rule(&self, rule_ptr: RulePtr) -> RuleItem {
        RuleItem::new(rule_ptr, self.symbols, self.rules)
    }
}
