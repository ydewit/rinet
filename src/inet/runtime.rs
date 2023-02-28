use rayon::{prelude::IntoParallelRefIterator, join};

use super::{rule::{RuleBook, RulePtr, Rule, RulePort, PortNum}, symbol::{SymbolBook, SymbolArity}, net::Net, equation::{EquationKind, EquationPtr, Equation}, cell::{Cell, CellPtr, PortKind, PortPtr}, var::{BVarPtr, VarPtr, VarKind, FVarPtr}, util::SmallVector, Polarity};

pub struct NewBVarPtrs {
    bvars: [BVarPtr; Self::MAX_BVARS_PER_RULE],
    len: usize
}

impl NewBVarPtrs {
    const MAX_BVARS_PER_RULE : usize = 10;

    fn new(net: &mut Net, len: usize) -> Self {
        assert!(len < Self::MAX_BVARS_PER_RULE);
        let mut this = Self { bvars: [BVarPtr::new(0);Self::MAX_BVARS_PER_RULE], len };
        for i in 0..len {
            this.bvars[i] = net.bvars.bvar();
        }
        this
    }

    fn get_bvar(&self, rule_bvar: BVarPtr) -> BVarPtr {
        self.bvars[rule_bvar.get_index()]
    }
}

pub struct Runtime<'a> {
    symbols: &'a SymbolBook,
    rules: &'a RuleBook,
    deferred : Vec<Equation>
}

impl<'a> Runtime<'a> {
    pub fn new(symbols: &'a SymbolBook, rules: &'a RuleBook) -> Self {
        Self { symbols, rules, deferred: Vec::new() }
    }

    pub fn eval(&mut self, mut net: Net) {
        let eqn_ptrs : Vec<EquationPtr> = net.equations().collect(); // TODO no need for EquationPtr, just use a simple Vec<Equation>
        for eqn_ptr in eqn_ptrs {
            let eqn = *net.get_equation(eqn_ptr);
            self.eval_equation(&mut net, eqn)
        }

        // loop {
        //     if self.deferred.len() == 0 {
        //         break;
        //     }

        //     for eqn in self.deferred.drain(..) {
        //         self.eval_equation(&mut net, eqn)
        //     }
        // }
    }

    fn eval_equation(&mut self, mut net: &mut Net, eqn: Equation) {
        println!("Evaluating equation: {:?}", eqn);
        match eqn.get_kind() {
            EquationKind::Redex => {
                self.eval_redex(&mut net, eqn.get_redex_ctr(), eqn.get_redex_fun());
            },
            EquationKind::Bind => {
                self.eval_bind(&mut net, eqn.get_bind_var(), eqn.get_bind_cell())
            },
            EquationKind::Connect => {
                self.eval_connect(&mut net, eqn.get_connect_left(), eqn.get_connect_right())
            },
        }
    }

    fn eval_redex(&mut self, mut net: &mut Net, ctr_ptr: CellPtr, fun_ptr: CellPtr) {
        let ctr = *net.get_cell(ctr_ptr);
        let fun = *net.get_cell(fun_ptr);

        println!("Evaluating redex: {} >< {}", self.symbols.get_name(ctr.get_symbol_ptr()), self.symbols.get_name(fun.get_symbol_ptr()));
        self.rewrite(&mut net, &ctr, &fun)
    }

    pub fn rewrite(&mut self, mut net: &mut Net, ctr: &Cell, fun: &Cell) {
        let rule_ptr = self.rules.get_by_symbols(ctr.get_symbol_ptr(), fun.get_symbol_ptr());
        let rule = self.rules.get_rule(rule_ptr);

        // preallocate bound vars (TODO can we allocate in consecutive indexes to simplify rewrite?)
        // let bvars = net.alloc_bvars(rule.get_bvar_count());
        let mut bvars = NewBVarPtrs::new(&mut net, rule.get_bvar_count());

        // interpret rule
        for rule_eqn_ptr in rule.body() {
            let rule_eqn = self.rules.get_equation(*rule_eqn_ptr);
            self.rewrite_equation(net, &mut bvars, ctr, fun, rule_eqn)
        }
    }

    fn rewrite_equation(&mut self, mut net: &mut Net, bvars: &mut NewBVarPtrs, ctr: &Cell, fun: &Cell, rule_eqn: &Equation) {
        match rule_eqn.get_kind() {
            EquationKind::Redex => {
                self.rewrite_redex(&mut net, bvars, ctr, fun, rule_eqn.get_redex_ctr(), rule_eqn.get_redex_fun())
            },
            EquationKind::Bind => {
                self.rewrite_bind(&mut net, bvars, ctr, fun, rule_eqn.get_bind_var(), rule_eqn.get_bind_cell());
            },
            EquationKind::Connect => {
                self.rewrite_connect(&mut net, bvars, ctr, fun, rule_eqn.get_connect_left(), rule_eqn.get_connect_right())
            },
        }
    }

    fn rewrite_redex(&mut self, net: &mut Net, bvars: &mut NewBVarPtrs, ctr: &Cell, fun: &Cell, rule_ctr_ptr: CellPtr, rule_fun_ptr: CellPtr) {
        let ctr_ptr = self.instantiate_cell(net, bvars, ctr, fun, rule_ctr_ptr);
        let fun_ptr = self.instantiate_cell(net, bvars, ctr, fun, rule_fun_ptr);

        //
        self.eval_equation(net, Equation::redex(ctr_ptr, fun_ptr));
    }

    fn rewrite_bind(&mut self, net: &mut Net, bvars: &mut NewBVarPtrs, ctr: &Cell, fun: &Cell, rule_var_ptr: VarPtr, rule_cell_ptr: CellPtr) {
        let cell_ptr = self.instantiate_cell(net, bvars, ctr, fun, rule_cell_ptr);
        match rule_var_ptr.get_kind() {
            VarKind::Bound => {
                let bvar_ptr = net.bvars.bvar();
                match cell_ptr.get_polarity() {
                    Polarity::Pos => {

                    },
                    Polarity::Neg => {

                    },
                }
            },
            // resolve free var against redex cells
            VarKind::Free => {
                let port_ptr = self.instantiate_port(net, bvars, ctr, fun, rule_var_ptr.into());
                match port_ptr.get_kind() {
                    PortKind::Cell => {
                        // new redex
                        match cell_ptr.get_polarity() {
                            Polarity::Pos => self.eval_equation(net, Equation::redex(cell_ptr, port_ptr.get_cell())),
                            Polarity::Neg => self.eval_equation(net, Equation::redex(port_ptr.get_cell(), cell_ptr))
                        }
                    },
                    PortKind::Var => {
                        match cell_ptr.get_polarity() {
                            Polarity::Pos => {
                                // match net.try_set_var(port_ptr.get_var(), cell_ptr){
                                //     // new redex (we got the )
                                //     Some(fun_ptr) => self.eval_equation(net, Equation::redex(cell_ptr, fun_ptr)),
                                //     None => () // var set, nothing else to do
                                // }
                            },
                            Polarity::Neg => {
                                // match net.try_get_var(port_ptr.get_var()) {
                                //     Some(ctr_ptr) => self.eval_equation(net, Equation::redex(ctr_ptr, cell_ptr)),
                                //     None => self.eval_equation(net, Equation::bind(port_ptr.get_var(), cell_ptr))
                                // }
                            }
                        }
                    }
                }
            }
        }
    }

    fn rewrite_connect(&mut self, net: &mut Net, bvars: &mut NewBVarPtrs, ctr: &Cell, fun: &Cell, rule_left_var: VarPtr, rule_right_var: VarPtr) {
        let left_port_ptr = self.instantiate_var(net, bvars, ctr, fun, rule_left_var);
        let right_port_ptr = self.instantiate_var(net, bvars, ctr, fun, rule_right_var);

        match (left_port_ptr.get_kind(), right_port_ptr.get_kind()) {
            (PortKind::Cell, PortKind::Cell) => {
                match left_port_ptr.get_cell().get_polarity() {
                    Polarity::Pos => self.eval_redex(net, left_port_ptr.get_cell(), right_port_ptr.get_cell()),
                    Polarity::Neg => self.eval_redex(net, right_port_ptr.get_cell(), left_port_ptr.get_cell())
                }
            },
            (PortKind::Cell, PortKind::Var) => {
                // bind
                net.equations.bind(right_port_ptr.get_var(), left_port_ptr.get_cell());
            },
            (PortKind::Var, PortKind::Cell) => {
                // bind
                net.equations.bind(left_port_ptr.get_var(), right_port_ptr.get_cell());
            },
            (PortKind::Var, PortKind::Var) => {
                // bind
                net.equations.connect(left_port_ptr.get_var(), right_port_ptr.get_var());
            },
        }
    }

    pub fn instantiate_cell(&mut self, net: &mut Net, bvars: &mut NewBVarPtrs, ctr: &Cell, fun: &Cell, rule_cell_ptr: CellPtr) -> CellPtr {
        let cell = self.rules.cells.get(rule_cell_ptr);
        println!("Instantiating cell: {}", self.symbols.get_name(cell.get_symbol_ptr()));
        match cell.get_symbol_ptr().get_arity() {
            SymbolArity::Zero => {
                net.cells.cell0(cell.get_symbol_ptr())
            },
            SymbolArity::One => {
                let port_ptr = self.instantiate_port(net, bvars, ctr, fun, cell.get_left_port());
                net.cells.cell1(cell.get_symbol_ptr(), port_ptr)
            },
            SymbolArity::Two => {
                let left_port_ptr = self.instantiate_port(net, bvars, ctr, fun, cell.get_left_port());
                let right_port_ptr = self.instantiate_port(net, bvars, ctr, fun, cell.get_right_port());

                net.cells.cell2(cell.get_symbol_ptr(), left_port_ptr, right_port_ptr)
            }
        }
    }

    fn instantiate_port(&mut self, mut net: &mut Net, bvars: &mut NewBVarPtrs, ctr: &Cell, fun: &Cell, rule_port_ptr: PortPtr) -> PortPtr {
        match rule_port_ptr.get_kind() {
            PortKind::Cell => {
                self.instantiate_cell(&mut net, bvars, ctr, fun, rule_port_ptr.get_cell()).into()
            },
            PortKind::Var => {
                self.instantiate_var(&mut net, bvars, ctr, fun, rule_port_ptr.get_var())
            },
        }
    }

    fn instantiate_var(&mut self, net: &mut Net, bvars: &mut NewBVarPtrs, ctr: &Cell, fun: &Cell, rule_var_ptr: VarPtr) -> PortPtr {
        match rule_var_ptr.get_kind() {
            VarKind::Bound => bvars.get_bvar(rule_var_ptr.into()).into(),
            VarKind::Free => self.resolve_fvar(ctr, fun, rule_var_ptr.into())
        }
    }

    fn resolve_fvar(&self, ctr: &Cell, fun: &Cell, ptr: FVarPtr) -> PortPtr {
        let fvar = self.rules.fvars.get(ptr);
        match fvar.get_store() {
            RulePort::Ctr { port : PortNum::Zero } => ctr.get_left_port(),
            RulePort::Ctr { port : PortNum::One } => ctr.get_right_port(),
            RulePort::Fun { port : PortNum::Zero } => fun.get_left_port(),
            RulePort::Fun { port : PortNum::One } => fun.get_right_port(),
        }
    }

    fn eval_bind(&mut self, net: &mut Net, var_ptr: VarPtr, cell_ptr: CellPtr) {
        println!("Evaluating bind: {:?} <- {:?}", var_ptr, cell_ptr);
        match var_ptr.get_kind() {
            VarKind::Bound => {
                match net.bvars.try_set(var_ptr.into(), cell_ptr) {
                    Some(other_cell_ptr) => {
                        match cell_ptr.get_polarity() {
                            Polarity::Pos => {
                                self.eval_equation(net, Equation::redex(cell_ptr, other_cell_ptr))
                            },
                            Polarity::Neg => {
                                self.eval_equation(net, Equation::redex(other_cell_ptr, cell_ptr))
                            },
                        }
                    },
                    None => () // set succeeded,
                }
            },
            VarKind::Free => {
                match cell_ptr.get_polarity() {
                    Polarity::Pos => net.fvars.send(var_ptr.into(), cell_ptr),
                    Polarity::Neg => todo!("wait or something else?")
                }
            },
        }
    }

    fn eval_connect(&mut self, net: &mut Net, left_var_ptr: VarPtr, right_var_ptr: VarPtr) {
        println!("Evaluating connect: {:?} <=> {:?}", left_var_ptr, right_var_ptr);
        match (left_var_ptr.get_kind(), right_var_ptr.get_kind()) {
            (VarKind::Bound, VarKind::Bound) => {
                let left_bvar = net.bvars.get(left_var_ptr.into());
                let right_bvar = net.bvars.get(right_var_ptr.into());
                todo!()
            },
            (VarKind::Bound, VarKind::Free) => {
                let left_bvar = net.bvars.get(left_var_ptr.into());
                let right_fvar = net.fvars.get(right_var_ptr.into());
                todo!()
            },
            (VarKind::Free, VarKind::Bound) => {
                let left_fvar = net.fvars.get(left_var_ptr.into());
                let right_bvar = net.bvars.get(right_var_ptr.into());
                todo!()
            },
            (VarKind::Free, VarKind::Free) => {
                let left_fvar = net.fvars.get(left_var_ptr.into());
                let right_fvar = net.fvars.get(right_var_ptr.into());
                todo!()
            },
        }
        // match (left_ptr, right_ptr){
        //     (None, None) => self.eval_equation(net, Equation::connect(left_var_ptr, left_var_ptr)),
        //     (None, Some(cell_ptr)) => self.eval_equation(net, Equation::bind(left_var_ptr, cell_ptr)),
        //     (Some(cell_ptr), None) => self.eval_equation(net, Equation::bind(right_var_ptr, cell_ptr)),
        //     (Some(left_cell_ptr), Some(right_cell_ptr)) => {
        //         match (left_cell_ptr.get_polarity(), right_cell_ptr.get_polarity()) {
        //             (Polarity::Pos, Polarity::Neg) => self.eval_equation(net, Equation::redex(left_cell_ptr, right_cell_ptr)),
        //             (Polarity::Neg, Polarity::Pos) => self.eval_equation(net, Equation::redex(right_cell_ptr, left_cell_ptr)),
        //             _ => panic!()
        //         }
        //     },
        // }
}


    // fn rewrite_cell<V: NetVisitor>(&self, visitor: V, cell_ptr: CellPtr) {
    //     let cell = self.get_cell(cell_ptr);
    //     match cell.get_symbol_ptr().get_arity() {
    //         SymbolArity::Zero => visitor.visit_cell0(cell_ptr, cell.get_symbol_ptr()),
    //         SymbolArity::One => {
    //             let port_ptr = cell.get_left_port();
    //             if visitor.visit_cell1(cell_ptr, cell.get_symbol_ptr(), port_ptr) {
    //                 self.rewrite_port(visitor, port_ptr);
    //             }
    //         },
    //         SymbolArity::Two => {
    //             let left_ptr = cell.get_left_port();
    //             let right_ptr = cell.get_right_port();
    //             if visitor.visit_cell2(cell_ptr, cell.get_symbol_ptr(), left_ptr, right_ptr) {
    //                 self.rewrite_port(visitor, left_ptr);
    //                 self.rewrite_port(visitor, right_ptr);
    //             }
    //         },
    //     }
    // }

    // fn rewrite_port<V: NetVisitor>(&self, visitor: V, port_ptr: PortPtr) {
    //     match port_ptr.get_kind() {
    //         PortKind::Cell => self.rewrite_cell(visitor, port_ptr.get_cell()),
    //         PortKind::Var => self.rewrite_var(visitor, port_ptr.get_var())
    //     }
    // }

    // fn rewrite_var<V: NetVisitor>(&self, visitor: V, var_ptr: VarPtr) {
    //     match var_ptr.is_free() {
    //         true => self.rewrite_fvar(visitor, var_ptr.into()),
    //         false => self.rewrite_bvar(visitor, var_ptr.into()),
    //     }
    // }

    // fn rewrite_fvar<V: NetVisitor>(&self, visitor: V, var_ptr: FVarPtr) {
    //     visitor.visit_fvar(var_ptr, self.get_fvar(var_ptr));
    // }

    // fn rewrite_bvar<V: NetVisitor>(&self, visitor: V, var_ptr: BVarPtr) {
    //     visitor.visit_bvar(var_ptr, self.get_bvar(var_ptr));
    // }



}

    // // let net = Arc::new(net);
    // match eqn.get_kind() {
    //     EquationKind::Redex => {
    //         // spawn(async {
    //             let _rule = todo!();
    //             let left_ptr = net.get_cell(eqn.get_redex_left());
    //             let right_ptr = net.get_cell(eqn.get_redex_right());

    //         // });
    //     },
    //     EquationKind::Bind => {
    //         let var = eqn.get_bind_var();
    //         let cell = eqn.get_bind_cell();
    //         match cell.get_polarity() {
    //             Polarity::Pos => {
    //                 if var.is_free() {
    //                     net.send(var.into(), cell);
    //                 }
    //                 else {
    //                     let cell = net.receive(var.into());

    //                 }
    //             },
    //             Polarity::Neg => todo!(),
    //         }
    //     },
    //     EquationKind::Connect => {
    //         panic!()
    //     },
    // }
