use crate::inet::{equation::order_ctr_fun, var::Var};

use super::{
    cell::{Cell, CellPtr},
    equation::{Equation, EquationKind},
    net::{Net, NetF},
    rule::{PortNum, RuleBook, RuleF, RulePort},
    symbol::{SymbolArity, SymbolBook},
    term::{TermKind, TermPtr},
    var::VarPtr,
};

pub struct BVarPtrs {
    bvar_ptrs: [VarPtr; Self::MAX_BVARS_PER_RULE as usize],
    len: u8,
}

impl BVarPtrs {
    const MAX_BVARS_PER_RULE: u8 = 10;

    fn new(net: &mut Net, len: u8) -> Self {
        assert!(len < Self::MAX_BVARS_PER_RULE);
        let mut this = Self {
            bvar_ptrs: [VarPtr::new(0); Self::MAX_BVARS_PER_RULE as usize],
            len,
        };
        for i in 0..len {
            this.bvar_ptrs[i as usize] = net.bvar();
        }
        this
    }

    fn get(&self, bvar_id: u8) -> VarPtr {
        self.bvar_ptrs[bvar_id as usize]
    }
}

// #[derive(Debug)]
// struct Equations {
//     eqns: [Equation<NetF>; 10],
//     len: u8,
// }

// impl Equations {
//     pub fn new() -> Self {
//         Self {
//             eqns: [Equation::<NetF>::from(0u64);10],
//             len: 0
//         }
//     }

//     pub fn iter(&self) -> std::slice::Iter<Equation<NetF>>  {
//         self.eqns[0..self.len as usize].iter()
//     }

//     pub fn push(&mut self, equation: Equation<NetF>) {
//         assert!(self.eqns.len() > self.len as usize);
//         self.eqns[self.len as usize] = equation;
//         self.len += 1
//     }

//     pub fn clean(&mut self) {
//         self.len = 0;
//     }
// }

pub struct Runtime<'a> {
    symbols: &'a SymbolBook,
    rules: &'a RuleBook<'a>,
}

impl<'a> Runtime<'a> {
    pub fn new(symbols: &'a SymbolBook, rules: &'a RuleBook) -> Self {
        Self { symbols, rules }
    }

    pub fn eval(&mut self, mut net: Net<'a>) -> Net<'a> {
        while net.body.len() > 0 {
            let eqns: Vec<Equation<NetF>> = net.body.drain_values().collect();
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
        println!(
            "Evaluating REDEX: {} ⋈ {}",
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

        // println!(
        //     "Found RULE: {} >< {}",
        //     self.symbols.get_name(rule.ctr),
        //     self.symbols.get_name(rule.fun)
        // );

        // preallocate bound vars (TODO can we allocate in consecutive indexes to simplify rewrite?)
        // let bvars = net.alloc_bvars(rule.get_bvar_count());
        let mut bvars = BVarPtrs::new(&mut net, rule.get_bvar_count());

        // interpret rule
        for rule_eqn_ptr in rule.body() {
            let rule_eqn = self.rules.get_equation(*rule_eqn_ptr).clone();
            net = self.rewrite_equation(net, &mut bvars, ctr, fun, rule_eqn);
        }

        net.heap.free_cell(ctr_ptr);
        net.heap.free_cell(fun_ptr);

        net
    }

    fn eval_bind(&mut self, mut net: Net<'a>, var_ptr: VarPtr, cell_ptr: CellPtr) -> Net<'a> {
        println!(
            "Evaluating BIND: {} <- {}",
            net.heap.display_var(self.symbols, var_ptr),
            net.heap.display_cell(self.symbols, cell_ptr)
        );

        let var = net.get_var(var_ptr);
        match var.get_store().get_or_set(cell_ptr) {
            Some(other_cell_ptr) => {
                if var.is_bound() {
                    // cell communicated, free the bound var
                    net.heap.free_var(var_ptr);
                }
                let (ctr_ptr, fun_ptr) = order_ctr_fun(cell_ptr, other_cell_ptr);
                net.body.alloc(Equation::redex(ctr_ptr, fun_ptr));
                net
            }
            None => net, // set succeeded,
        }
    }

    fn eval_connect(
        &mut self,
        net: Net<'a>,
        left_var_ptr: VarPtr,
        right_var_ptr: VarPtr,
    ) -> Net<'a> {
        println!(
            "Evaluating CONNECT: {} <-> {}",
            net.heap.display_var(self.symbols, left_var_ptr),
            net.heap.display_var(self.symbols, right_var_ptr)
        );

        let left_var = net.heap.get_var(left_var_ptr).unwrap();
        let right_var = net.heap.get_var(right_var_ptr).unwrap();

        todo!()
    }

    fn rewrite_equation(
        &mut self,
        net: Net<'a>,
        bvars: &mut BVarPtrs,
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
                rule_eqn.get_bind_var(),
                rule_eqn.get_bind_cell(),
            ),
            EquationKind::Connect => self.instantiate_connect(
                net,
                bvars,
                ctr,
                fun,
                rule_eqn.get_connect_left(),
                rule_eqn.get_connect_right(),
            ),
        }
    }

    fn instantiate_redex(
        &mut self,
        mut net: Net<'a>,
        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_ctr_ptr: CellPtr,
        rule_fun_ptr: CellPtr,
    ) -> Net<'a> {
        let ctr_ptr = self.instantiate_cell(&mut net, bvars, ctr, fun, rule_ctr_ptr);
        let fun_ptr = self.instantiate_cell(&mut net, bvars, ctr, fun, rule_fun_ptr);

        println!(
            "Instantiate redex: {} = {}  ⟶  {} = {}",
            self.rules.display_cell(rule_ctr_ptr),
            self.rules.display_cell(rule_fun_ptr),
            net.heap.display_cell(self.symbols, ctr_ptr),
            net.heap.display_cell(self.symbols, fun_ptr)
        );

        net.body.alloc(Equation::redex(ctr_ptr, fun_ptr));
        net
    }

    fn instantiate_bind(
        &mut self,
        mut net: Net<'a>,
        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_var_ptr: VarPtr,
        rule_cell_ptr: CellPtr,
    ) -> Net<'a> {
        let cell_ptr = self.instantiate_cell(&mut net, bvars, ctr, fun, rule_cell_ptr);
        let term_ptr = self.instantiate_var(&net, bvars, ctr, fun, rule_var_ptr);

        print!(
            "Instantiate bind: {} ← {}",
            self.rules.display_var(rule_var_ptr),
            self.rules.display_cell(rule_cell_ptr)
        );

        match term_ptr.get_kind() {
            TermKind::Cell => {
                let (ctr_ptr, fun_ptr) = order_ctr_fun(cell_ptr, term_ptr.get_cell_ptr());
                println!(
                    "  ⟶  {} = {}",
                    net.heap.display_cell(self.symbols, ctr_ptr),
                    net.heap.display_cell(self.symbols, fun_ptr)
                );

                net.body.alloc(Equation::redex(ctr_ptr, fun_ptr));
            }
            TermKind::Var => {
                let var = net.get_var(term_ptr.get_var_ptr());
                match var.get_store().get_or_set(cell_ptr) {
                    Some(other_cell_ptr) => {
                        let (ctr_ptr, fun_ptr) = order_ctr_fun(cell_ptr, other_cell_ptr);
                        println!(
                            "  ⟶  {} = {}",
                            net.heap.display_cell(self.symbols, ctr_ptr),
                            net.heap.display_cell(self.symbols, fun_ptr)
                        );
                        net.body.alloc(Equation::redex(ctr_ptr, fun_ptr));
                    }
                    None => {
                        println!(
                            "  ⟶  {} ← {}",
                            net.heap.display_var(self.symbols, term_ptr.get_var_ptr()),
                            net.heap.display_cell(self.symbols, cell_ptr)
                        );
                    }
                }
            }
        }
        net
    }

    fn instantiate_connect(
        &mut self,
        mut net: Net<'a>,

        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_left_var: VarPtr,
        rule_right_var: VarPtr,
    ) -> Net<'a> {
        let left_port_ptr = self.instantiate_var(&net, bvars, ctr, fun, rule_left_var);
        let right_port_ptr = self.instantiate_var(&net, bvars, ctr, fun, rule_right_var);

        print!(
            "Instantiate connect: {} ↔ {}",
            self.rules.display_var(rule_left_var),
            self.rules.display_var(rule_right_var)
        );

        match (left_port_ptr.get_kind(), right_port_ptr.get_kind()) {
            (TermKind::Cell, TermKind::Cell) => {
                let (ctr_ptr, fun_ptr) =
                    order_ctr_fun(left_port_ptr.get_cell_ptr(), right_port_ptr.get_cell_ptr());
                println!(
                    "  ⟶  {} = {}",
                    net.heap.display_cell(self.symbols, ctr_ptr),
                    net.heap.display_cell(self.symbols, fun_ptr)
                );
                net.body.alloc(Equation::redex(ctr_ptr, fun_ptr));
            }
            (TermKind::Cell, TermKind::Var) => {
                // bind
                println!(
                    "  ⟶  {} ← {}",
                    net.heap
                        .display_var(self.symbols, right_port_ptr.get_var_ptr()),
                    net.heap
                        .display_cell(self.symbols, left_port_ptr.get_cell_ptr())
                );
                net.body.alloc(Equation::bind(
                    right_port_ptr.get_var_ptr(),
                    left_port_ptr.get_cell_ptr(),
                ));
            }
            (TermKind::Var, TermKind::Cell) => {
                // bind
                println!(
                    "  ⟶  {} ← {}",
                    net.heap
                        .display_var(self.symbols, left_port_ptr.get_var_ptr()),
                    net.heap
                        .display_cell(self.symbols, right_port_ptr.get_cell_ptr())
                );
                net.body.alloc(Equation::bind(
                    left_port_ptr.get_var_ptr(),
                    right_port_ptr.get_cell_ptr(),
                ));
            }
            (TermKind::Var, TermKind::Var) => {
                // connect
                println!(
                    "  ⟶  {} ↔ {}",
                    net.heap
                        .display_var(self.symbols, right_port_ptr.get_var_ptr()),
                    net.heap
                        .display_var(self.symbols, right_port_ptr.get_var_ptr())
                );
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
        cell_ptr
    }

    fn instantiate_port(
        &mut self,
        mut net: &mut Net<'a>,
        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_port_ptr: TermPtr,
    ) -> TermPtr {
        match rule_port_ptr.get_kind() {
            TermKind::Cell => self
                .instantiate_cell(&mut net, bvars, ctr, fun, rule_port_ptr.get_cell_ptr())
                .into(),
            TermKind::Var => {
                self.instantiate_var(net, bvars, ctr, fun, rule_port_ptr.get_var_ptr())
            }
        }
    }

    fn instantiate_var(
        &mut self,
        net: &Net<'a>,
        bvars: &mut BVarPtrs,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_var_ptr: VarPtr,
    ) -> TermPtr {
        let rule_var = self.rules.heap.get_var(rule_var_ptr).unwrap();
        match rule_var {
            Var::Bound(bvar_id) => bvars.get(*bvar_id).into(),
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
}
