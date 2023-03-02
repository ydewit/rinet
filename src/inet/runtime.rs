use super::{
    cell::{Cell, CellPtr},
    equation::{Equation, EquationKind},
    net::{Net, NetF},
    rule::{PortNum, RuleBook, RuleF, RulePort, RuleStore},
    symbol::{SymbolArity, SymbolBook},
    term::{TermKind, TermPtr},
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
            this.bvar_ptrs[i] = net.var();
        }
        this
    }

    fn get(&self, bvar_ptr: VarPtr) -> VarPtr {
        self.bvar_ptrs[bvar_ptr.get_index()]
    }
}

pub struct Runtime<'a> {
    symbols: &'a SymbolBook,
    rules: &'a RuleBook
}

impl<'a> Runtime<'a> {
    pub fn new(symbols: &'a SymbolBook, rules: &'a RuleBook) -> Self {
        Self { symbols, rules }
    }

    pub fn eval(&mut self, mut net: Net) -> Net {
        let eqns: Vec<Equation<NetF>> = net.body.drain_values().collect();
        for eqn in eqns {
            net = self.eval_equation(net, eqn)
        }
        net
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
            net.heap.display_cell(self.symbols, ctr_ptr),
            net.heap.display_cell(self.symbols, fun_ptr)
        );

        let ctr = *net.get_cell(ctr_ptr);
        let fun = *net.get_cell(fun_ptr);

        // find rule
        let rule_ptr = self
            .rules
            .get_by_symbols(ctr.get_symbol_ptr(), fun.get_symbol_ptr());
        let rule = self.rules.get_rule(rule_ptr);

        println!("Found RULE: {}", self.rules.display_rule(self.symbols, rule_ptr));

        // preallocate bound vars (TODO can we allocate in consecutive indexes to simplify rewrite?)
        // let bvars = net.alloc_bvars(rule.get_bvar_count());
        let mut bvars = BVarPtrs::new(&mut net, rule.get_bvar_count());

        // interpret rule
        for rule_eqn_ptr in rule.body() {
            let rule_eqn = self.rules.get_equation(*rule_eqn_ptr).clone();
            net = self.rewrite_rule_equation(net, &mut bvars, ctr, fun, rule_eqn)

            // net.cells.free(ctr_ptr);
            // net.cells.free(fun_ptr);
        }
        net
    }

    fn eval_bind(&mut self, mut net: Net, var_ptr: VarPtr, cell_ptr: CellPtr) -> Net {
        println!(
            "Evaluating BIND: {} <- {}",
            net.heap.display_var(self.symbols, var_ptr),
            net.heap.display_cell(self.symbols, cell_ptr)
        );

        match net.try_set_var(var_ptr, cell_ptr) {
            Some(other_cell_ptr) => {
                // cell communicated, free the bound var
                net.heap.free_var(var_ptr);
                match cell_ptr.get_polarity() {
                    Polarity::Pos => self.eval_redex(net, cell_ptr, other_cell_ptr),
                    Polarity::Neg => self.eval_redex(net, other_cell_ptr, cell_ptr),
                }
            }
            None => net, // set succeeded,
        }
    }

    fn eval_connect(&mut self, net: Net, left_var_ptr: VarPtr, right_var_ptr: VarPtr) -> Net {
        println!(
            "Evaluating CONNECT: {} <-> {}",
            net.heap.display_var(self.symbols, left_var_ptr),
            net.heap.display_var(self.symbols, right_var_ptr)
        );

        let left_var = net.heap.get_var(left_var_ptr).unwrap();
        let right_var = net.heap.get_var(right_var_ptr).unwrap();

        todo!()
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
        let term_ptr = self.instantiate_var(&net, bvars, ctr, fun, rule_var_ptr);
        match term_ptr.get_kind() {
            TermKind::Cell => match cell_ptr.get_polarity() {
                Polarity::Pos => self.eval_redex(net, cell_ptr, term_ptr.get_cell()),
                Polarity::Neg => self.eval_redex(net, term_ptr.get_cell(), cell_ptr),
            },
            TermKind::Var => {
                match net.try_set_var(term_ptr.get_var(), cell_ptr) {
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
            (TermKind::Cell, TermKind::Cell) => match left_port_ptr.get_cell().get_polarity() {
                Polarity::Pos => {
                    self.eval_redex(net, left_port_ptr.get_cell(), right_port_ptr.get_cell())
                }
                Polarity::Neg => {
                    self.eval_redex(net, right_port_ptr.get_cell(), left_port_ptr.get_cell())
                }
            },
            (TermKind::Cell, TermKind::Var) => {
                // bind
                net.bind(right_port_ptr.get_var(), left_port_ptr.get_cell());
                net
            }
            (TermKind::Var, TermKind::Cell) => {
                // bind
                net.bind(left_port_ptr.get_var(), right_port_ptr.get_cell());
                net
            }
            (TermKind::Var, TermKind::Var) => {
                // bind
                net.connect(left_port_ptr.get_var(), right_port_ptr.get_var());
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
        let rule_cell = self.rules.heap.get_cell(rule_cell_ptr).unwrap();
        let cell_ptr = match rule_cell.get_symbol_ptr().get_arity() {
            SymbolArity::Zero => net.heap.cell0(rule_cell.get_symbol_ptr()),
            SymbolArity::One => {
                let term_ptr =
                    self.instantiate_port(net, bvars, ctr, fun, rule_cell.get_left_port());
                net.heap.cell1(rule_cell.get_symbol_ptr(), term_ptr)
            }
            SymbolArity::Two => {
                let left_port_ptr =
                    self.instantiate_port(net, bvars, ctr, fun, rule_cell.get_left_port());
                let right_port_ptr =
                    self.instantiate_port(net, bvars, ctr, fun, rule_cell.get_right_port());

                net.heap
                    .cell2(rule_cell.get_symbol_ptr(), left_port_ptr, right_port_ptr)
            }
        };
        println!(
            "Instantiating cell: {} --> {}",
            self.rules.display_cell(self.symbols, rule_cell_ptr),
            net.heap.display_cell(self.symbols, cell_ptr)
        );
        cell_ptr
    }

    fn instantiate_port(
        &mut self,
        mut net: &mut Net,
        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_port_ptr: TermPtr,
    ) -> TermPtr {
        match rule_port_ptr.get_kind() {
            TermKind::Cell => self
                .instantiate_cell(&mut net, bvars, ctr, fun, rule_port_ptr.get_cell())
                .into(),
            TermKind::Var => self.instantiate_var(net, bvars, ctr, fun, rule_port_ptr.get_var()),
        }
    }

    fn instantiate_var(
        &mut self,
        net: &Net,
        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_var_ptr: VarPtr,
    ) -> TermPtr {
        print!(
            "Instantiating var: {}",
            self.rules.display_var(self.symbols, rule_var_ptr)
        );
        let rule_var = self.rules.heap.get_var(rule_var_ptr).unwrap();
        let term_ptr = match &rule_var.0 {
            RuleStore::Bound => bvars.get(rule_var_ptr).into(),
            RuleStore::Free { port } => self.resolve_fvar(ctr, fun, *port),
        };
        println!(" --> {}", net.heap.display_term(self.symbols, term_ptr));
        term_ptr
    }

    fn resolve_fvar(&self, ctr: Cell<NetF>, fun: Cell<NetF>, port: RulePort) -> TermPtr {
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
}
