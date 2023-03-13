use crate::inet::var::{PVarPtrBuffer, Var};

use super::{
    cell::{Cell, CellPtr},
    equation::{Equation, EquationKind},
    net::{Net, NetF},
    rule::{PortNum, RuleBook, RuleF, RulePort, Rule},
    symbol::{SymbolArity, SymbolBook},
    term::{TermKind, TermPtr},
    var::PVarPtr,
    Polarity,
};

pub struct Runtime<'a> {
    debug: bool,
    symbols: &'a SymbolBook,
    rules: &'a RuleBook<'a>,
}

impl<'a> Runtime<'a> {
    pub fn new(symbols: &'a SymbolBook, rules: &'a RuleBook, debug: bool) -> Self {
        Self {
            symbols,
            rules,
            debug,
        }
    }

    pub fn eval(&mut self, mut net: Net<'a>) -> Net<'a> {
        while net.body.len() > 0 {
            println!("Equations: {}, Cells: {}, Vars: {}", net.body.len(), net.heap.cells.len(), net.heap.vars.len());
            // println!("{}", net);
            let eqns: Vec<Equation<NetF>> = net.body.drain_values().collect();
            println!("Equations: {}, Cells: {}, Vars: {}", net.body.len(), net.heap.cells.len(), net.heap.vars.len());
            for eqn in eqns {
                net = self.eval_equation(net, eqn);
            }
        }
        net
    }

    fn eval_equation(&mut self, net: Net<'a>, eqn: Equation<NetF>) -> Net<'a> {
        match eqn.get_kind() {
            EquationKind::Redex => self.eval_redex(net, eqn.get_redex_ctr(), eqn.get_redex_fun()),
            EquationKind::Bind => self.eval_bind(net, eqn.get_bind_var(), eqn.get_bind_cell()),
            EquationKind::Connect => {
                self.eval_connect(net, eqn.get_connect_left(), eqn.get_connect_right())
            }
        }
    }

    fn eval_redex(&mut self, mut net: Net<'a>, ctr_ptr: CellPtr, fun_ptr: CellPtr) -> Net<'a> {
        if self.debug {
            println!(
                "Evaluating REDEX: {} ⋈ {}",
                net.display_cell(ctr_ptr),
                net.display_cell(fun_ptr)
            );
        }

        let ctr = net.free_cell(ctr_ptr).unwrap();
        let fun = net.free_cell(fun_ptr).unwrap();

        // find rule
        let rule_ptr = self
        .rules
        .get_by_symbols(ctr.get_symbol_ptr(), fun.get_symbol_ptr())
        .or_else(|| {
            panic!(
                "Rule not found for: {} >< {}",
                self.symbols.get_name(ctr.get_symbol_ptr()).unwrap(),
                self.symbols.get_name(fun.get_symbol_ptr()).unwrap()
            )
        })
        .unwrap();
        let rule = self.rules.get_rule(rule_ptr);

        // preallocate bound vars (TODO can we allocate in consecutive indexes to simplify rewrite?)
        // let bvars = net.alloc_bvars(rule.get_bvar_count());
        let mut bvars = self.new_bvar_buffer(&mut net, rule.get_bvar_count());

        // interpret rule
        for rule_eqn_ptr in rule.body() {
            let rule_eqn = self.rules.get_equation(*rule_eqn_ptr).clone();
            net = self.rewrite_equation(net, &mut bvars, ctr, fun, rule_eqn);
        }

        net
    }

    fn eval_bind(&mut self, mut net: Net<'a>, var_ptr: PVarPtr, cell_ptr: CellPtr) -> Net<'a> {
        if self.debug {
            println!(
                "Evaluating BIND: {} <- {}",
                net.heap.display_var(self.symbols, (&var_ptr).into()),
                net.heap.display_cell(self.symbols, cell_ptr.into())
            );
        }

        let var = net.get_var(&var_ptr);
        match var.get_store().get_or_set(cell_ptr) {
            Some(other_cell_ptr) => {
                if var.is_bound() {
                    // cell communicated, free the bound var
                    net.heap.free_var(&var_ptr);
                }
                let (ctr_ptr, fun_ptr) = self.order_ctr_fun(&net, cell_ptr, other_cell_ptr);
                net.body.alloc(Equation::redex(ctr_ptr, fun_ptr));
                net
            }
            None => net, // set succeeded,
        }
    }

    fn eval_connect(
        &mut self,
        mut net: Net<'a>,
        left_var_ptr: PVarPtr,
        right_var_ptr: PVarPtr,
    ) -> Net<'a> {
        if self.debug {
            println!(
                "Evaluating CONNECT: {} ↔ {}",
                net.heap.display_var(self.symbols, (&left_var_ptr).into()),
                net.heap.display_var(self.symbols, (&right_var_ptr).into())
            );
        }

        let left_var = net.heap.get_var((&left_var_ptr).into()).unwrap();
        let right_var = net.heap.get_var((&right_var_ptr).into()).unwrap();

        match (
            left_var.get_store().get_cell_ptr(),
            right_var.get_store().get_cell_ptr(),
        ) {
            (Some(left_cell_ptr), Some(right_cell_ptr)) => {
                let (left_cell_ptr, right_cell_ptr) =
                    self.order_ctr_fun(&net, left_cell_ptr, right_cell_ptr);
                net.body
                    .alloc(Equation::redex(left_cell_ptr, right_cell_ptr));
            },
            _ => () // do nothing
            // (None, None) => todo!(),
            // (None, Some(cell_ptr)) => {
            //     net.body.alloc(Equation::bind(left_var_ptr, cell_ptr));
            // }
            // (Some(cell_ptr), None) => {
            //     net.body.alloc(Equation::bind(right_var_ptr, cell_ptr));
            // }
        }
        net
    }

    fn rewrite_equation(
        &mut self,
        net: Net<'a>,
        bvars: &mut PVarPtrBuffer,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_eqn: Equation<RuleF>,
    ) -> Net<'a> {
        match rule_eqn.get_kind() {
            EquationKind::Redex => self.instantiate_redex(
                net,
                bvars,
                ctr,
                fun,
                rule_eqn.get_redex_ctr(),
                rule_eqn.get_redex_fun(),
            ),
            EquationKind::Bind => self.instantiate_bind(
                net,
                bvars,
                ctr,
                fun,
                &rule_eqn.get_bind_var(),
                rule_eqn.get_bind_cell(),
            ),
            EquationKind::Connect => self.instantiate_connect(
                net,
                bvars,
                ctr,
                fun,
                &rule_eqn.get_connect_left(),
                &rule_eqn.get_connect_right(),
            ),
        }
    }

    fn instantiate_redex(
        &mut self,
        mut net: Net<'a>,
        bvars: &mut PVarPtrBuffer,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_ctr_ptr: CellPtr,
        rule_fun_ptr: CellPtr,
    ) -> Net<'a> {
        let ctr_ptr = self.instantiate_cell(&mut net, bvars, ctr, fun, rule_ctr_ptr);
        let fun_ptr = self.instantiate_cell(&mut net, bvars, ctr, fun, rule_fun_ptr);

        if self.debug {
            println!(
                "Instantiate rule redex: {} = {}  ⟶  {} = {}",
                self.rules.display_cell(rule_ctr_ptr),
                self.rules.display_cell(rule_fun_ptr),
                net.heap.display_cell(self.symbols, ctr_ptr),
                net.heap.display_cell(self.symbols, fun_ptr)
            );
        }
        net.body.alloc(Equation::redex(ctr_ptr, fun_ptr));
        net
    }

    fn instantiate_bind(
        &mut self,
        mut net: Net<'a>,
        bvars: &mut PVarPtrBuffer,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_var_ptr: &PVarPtr,
        rule_cell_ptr: CellPtr,
    ) -> Net<'a> {
        let cell_ptr = self.instantiate_cell(&mut net, bvars, ctr, fun, rule_cell_ptr);
        let term_ptr = self.instantiate_var(&net, bvars, ctr, fun, rule_var_ptr);

        match term_ptr.get_kind() {
            TermKind::Cell => {
                if self.debug {
                    print!(
                        "Instantiate rule bind: {} ← {}",
                        self.rules.display_var(rule_var_ptr.into()),
                        self.rules.display_cell(rule_cell_ptr)
                    );
                }

                let (ctr_ptr, fun_ptr) =
                    self.order_ctr_fun(&net, cell_ptr, term_ptr.get_cell_ptr());

                if self.debug {
                    println!(
                        "  ⟶  {} = {}",
                        net.heap.display_cell(self.symbols, ctr_ptr),
                        net.heap.display_cell(self.symbols, fun_ptr)
                    );
                }
                net.body.alloc(Equation::redex(ctr_ptr, fun_ptr));
            }
            TermKind::Var => {
                let var = net.get_var(&term_ptr.get_var_ptr());
                match var.get_store().get_or_set(cell_ptr) {
                    Some(other_cell_ptr) => {
                        if self.debug {
                            print!(
                                "Instantiate rule bind: {}[{}] ← {}",
                                self.rules.display_var(rule_var_ptr.get_fvar_ptr()),
                                net.heap.display_cell(self.symbols, other_cell_ptr),
                                self.rules.display_cell(rule_cell_ptr)
                            );
                        }
                        let (ctr_ptr, fun_ptr) = self.order_ctr_fun(&net, cell_ptr, other_cell_ptr);
                        if self.debug {
                            println!(
                                "  ⟶  {} = {}",
                                net.heap.display_cell(self.symbols, ctr_ptr),
                                net.heap.display_cell(self.symbols, fun_ptr)
                            );
                        }
                        net.body.alloc(Equation::redex(ctr_ptr, fun_ptr));
                    }
                    None => {
                        if self.debug {
                            print!(
                                "Instantiate rule bind: {} ← {}",
                                self.rules.display_var(rule_var_ptr.get_fvar_ptr()),
                                self.rules.display_cell(rule_cell_ptr)
                            );
                            println!(
                                "  ⟶  {} ← {}",
                                net.heap
                                    .display_var(self.symbols, term_ptr.get_var_ptr().into()),
                                net.heap.display_cell(self.symbols, cell_ptr)
                            );
                        }
                    }
                }
            }
        }
        net
    }

    fn instantiate_connect(
        &mut self,
        mut net: Net<'a>,

        bvars: &mut PVarPtrBuffer,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_left_var: &PVarPtr,
        rule_right_var: &PVarPtr,
    ) -> Net<'a> {
        let left_port_ptr = self.instantiate_var(&net, bvars, ctr, fun, rule_left_var);
        let right_port_ptr = self.instantiate_var(&net, bvars, ctr, fun, rule_right_var);

        if self.debug {
            print!(
                "Instantiate rule connect: {} ↔ {}",
                self.rules.display_var(rule_left_var.into()),
                self.rules.display_var(rule_right_var.into())
            );
        }

        match (left_port_ptr.get_kind(), right_port_ptr.get_kind()) {
            (TermKind::Cell, TermKind::Cell) => {
                let (ctr_ptr, fun_ptr) = self.order_ctr_fun(
                    &net,
                    left_port_ptr.get_cell_ptr(),
                    right_port_ptr.get_cell_ptr(),
                );
                if self.debug {
                    println!(
                        "  ⟶  {} = {}",
                        net.heap.display_cell(self.symbols, ctr_ptr),
                        net.heap.display_cell(self.symbols, fun_ptr)
                    );
                }
                net.body.alloc(Equation::redex(ctr_ptr, fun_ptr));
            }
            (TermKind::Cell, TermKind::Var) => {
                // bind
                if self.debug {
                    println!(
                        "  ⟶  {} ← {}",
                        net.heap
                            .display_var(self.symbols, right_port_ptr.get_var_ptr().into()),
                        net.heap
                            .display_cell(self.symbols, left_port_ptr.get_cell_ptr())
                    );
                }
                net.body.alloc(Equation::bind(
                    right_port_ptr.get_var_ptr().into(),
                    left_port_ptr.get_cell_ptr(),
                ));
            }
            (TermKind::Var, TermKind::Cell) => {
                // bind
                if self.debug {
                    println!(
                        "  ⟶  {} ← {}",
                        net.heap
                            .display_var(self.symbols, left_port_ptr.get_var_ptr().into()),
                        net.heap
                            .display_cell(self.symbols, right_port_ptr.get_cell_ptr())
                    );
                }
                net.body.alloc(Equation::bind(
                    left_port_ptr.get_var_ptr().into(),
                    right_port_ptr.get_cell_ptr(),
                ));
            }
            (TermKind::Var, TermKind::Var) => {
                // connect
                if self.debug {
                    println!(
                        "  ⟶  {} ↔ {}",
                        net.heap
                            .display_var(self.symbols, right_port_ptr.get_var_ptr().into()),
                        net.heap
                            .display_var(self.symbols, right_port_ptr.get_var_ptr().into())
                    );
                }
                net.body.alloc(Equation::connect(
                    left_port_ptr.get_var_ptr(),
                    right_port_ptr.get_var_ptr(),
                ));
            }
        }
        net
    }

    pub fn instantiate_cell(
        &mut self,
        net: &mut Net<'a>,
        bvars: &mut PVarPtrBuffer,
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
        cell_ptr
    }

    fn instantiate_port(
        &mut self,
        mut net: &mut Net<'a>,
        bvars: &mut PVarPtrBuffer,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_port_ptr: TermPtr,
    ) -> TermPtr {
        match rule_port_ptr.get_kind() {
            TermKind::Cell => self
                .instantiate_cell(&mut net, bvars, ctr, fun, rule_port_ptr.get_cell_ptr())
                .into(),
            TermKind::Var => {
                self.instantiate_var(net, bvars, ctr, fun, &rule_port_ptr.get_var_ptr())
            }
        }
    }

    fn instantiate_var(
        &mut self,
        net: &Net<'a>,
        bvars: &mut PVarPtrBuffer,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_var_ptr: &PVarPtr,
    ) -> TermPtr {
        let var_polarity = rule_var_ptr.get_polarity();
        let rule_var = self.rules.heap.get_var(rule_var_ptr.into()).unwrap();
        match rule_var {
            Var::Bound(bvar_id) => match var_polarity {
                Polarity::Pos => bvars.get_pos_var(*bvar_id).into(),
                Polarity::Neg => bvars.get_neg_var(*bvar_id).into(),
            },
            Var::Free(port) => self.resolve_fvar(ctr, fun, *port),
        }
    }

    fn resolve_fvar(&self, ctr: Cell<NetF>, fun: Cell<NetF>, port: RulePort) -> TermPtr {
        match port {
            RulePort::Ctr(PortNum::Zero) => ctr.get_left_port(),
            RulePort::Ctr(PortNum::One) => ctr.get_right_port(),
            RulePort::Fun(PortNum::Zero) => fun.get_left_port(),
            RulePort::Fun(PortNum::One) => fun.get_right_port(),
        }
    }

    fn order_ctr_fun(
        &self,
        net: &Net,
        left_ptr: CellPtr,
        right_ptr: CellPtr,
    ) -> (CellPtr, CellPtr) {
        match (left_ptr.get_polarity(), right_ptr.get_polarity()) {
            (Polarity::Pos, Polarity::Neg) => (left_ptr, right_ptr),
            (Polarity::Neg, Polarity::Pos) => (right_ptr, left_ptr),
            (Polarity::Neg, Polarity::Neg) => panic!(
                "Short-circuit (Neg x Neg): {} x {} ({:?} x {:?})",
                net.display_cell(left_ptr),
                net.display_cell(right_ptr),
                left_ptr,
                right_ptr
            ),
            (Polarity::Pos, Polarity::Pos) => panic!(
                "Short-circuit (Pos x Pos): {} x {} ({:?} x {:?})",
                net.display_cell(left_ptr),
                net.display_cell(right_ptr),
                left_ptr,
                right_ptr
            ),
        }
    }

    pub(crate) fn new_bvar_buffer(&self, net: &mut Net, bvar_count: u8) -> PVarPtrBuffer {
        let mut buffer = PVarPtrBuffer::new(bvar_count);
        for i in 0..bvar_count {
            buffer.set(i, net.bvar())
        }
        buffer
    }
}
