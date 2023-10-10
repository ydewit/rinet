use std::{thread, sync::atomic::AtomicUsize};

use crate::inet::var::{PVarPtrBuffer, Var};

use rayon::{
    prelude::{ParallelDrainRange, ParallelIterator},
    Scope,
};
use tracing::debug;

use super::{
    cell::{Cell, CellPtr},
    equation::{Equation, EquationKind},
    heap::Heap,
    net::{Net, NetF, NetStore},
    rule::{RuleF, RulePort, RuleSet},
    symbol::{SymbolArity, SymbolBook},
    term::{TermKind, TermPtr},
    var::PVarPtr,
    Polarity,
};

#[derive (Debug)]
pub struct Runtime<'a> {
    debug: bool,
    rules: &'a RuleSet<'a>,
    rewrites: AtomicUsize
}

impl<'a> Runtime<'a> {
    pub fn new(rules: &'a RuleSet, debug: bool) -> Self {
        Self { rules, debug, rewrites: Default::default() }
    }

    pub fn get_rewrites(&self) -> usize {
        self.rewrites.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[tracing::instrument]
    pub fn eval(&self, mut net: Net<'a>) -> Net<'a> {
        rayon::scope(|scope| {
            net.body
                .par_drain(..)
                .for_each(|eqn| self.eval_equation(scope, &net.symbols, &net.heap, eqn));
        });
        net
    }

    fn eval_equation<'scope>(
        &'scope self,
        scope: &Scope<'scope>,
        symbols: &'scope SymbolBook,
        heap: &'scope Heap<NetF>,
        eqn: Equation<NetF>,
    ) {
        match eqn.get_kind() {
            EquationKind::Redex => self.eval_redex(
                scope,
                symbols,
                heap,
                eqn.get_redex_ctr(),
                eqn.get_redex_fun(),
            ),
            EquationKind::Bind => self.eval_bind(
                scope,
                symbols,
                heap,
                eqn.get_bind_var(),
                eqn.get_bind_cell(),
            ),
            EquationKind::Connect => self.eval_connect(
                scope,
                symbols,
                heap,
                eqn.get_connect_left(),
                eqn.get_connect_right(),
            ),
        }
    }

    fn eval_redex<'scope>(
        &'scope self,
        scope: &Scope<'scope>,
        symbols: &'scope SymbolBook,
        heap: &'scope Heap<NetF>,
        ctr_ptr: CellPtr,
        fun_ptr: CellPtr,
    ) {
        debug!(
            "[{}] Evaluating REDEX: >>>>  {} = {}  <<<<",
            rayon::current_thread_index().unwrap(),
            heap.display_cell(symbols, &ctr_ptr),
            heap.display_cell(symbols, &fun_ptr)
        );

        let ctr = heap.free_cell(ctr_ptr);
        let fun = heap.free_cell(fun_ptr);

        // find rule
        let rule_ptr = self
            .rules
            .get_by_symbols(ctr.get_symbol_ptr(), fun.get_symbol_ptr())
            .or_else(|| {
                panic!(
                    "Rule not found for: {} ⋈ {}",
                    symbols.get_name(ctr.get_symbol_ptr()).unwrap(),
                    symbols.get_name(fun.get_symbol_ptr()).unwrap()
                )
            })
            .unwrap();
        let rule = self.rules.get_rule(rule_ptr);

        self.rewrites.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        // info!("Rule: {}", rule.display(symbols, heap));
        // preallocate bound vars (TODO can we allocate in consecutive indexes to simplify rewrite?)
        // let bvars = net.alloc_bvars(rule.get_bvar_count());
        let mut bvars = self.new_bvar_buffer(symbols, heap, rule.get_bvar_count());

        // interpret rule
        for rule_eqn_ptr in rule.body() {
            let rule_eqn = self.rules.get_equation(*rule_eqn_ptr).clone();
            self.rewrite_equation(scope, symbols, heap, &mut bvars, ctr, fun, rule_eqn);
        }
    }

    fn eval_bind<'scope>(
        &'scope self,
        scope: &Scope<'scope>,
        symbols: &'scope SymbolBook,
        heap: &'scope Heap<NetF>,
        var_ptr: PVarPtr,
        cell_ptr: CellPtr,
    ) {
        let var = heap.get_var(&var_ptr);
        match var.get_store().set_or_get(cell_ptr) {
            (cell_ptr, Some(other_cell_ptr)) => {
                if var.is_bound() {
                    // cell communicated, free the bound var
                    heap.free_var(var_ptr);
                }
                let (ctr_ptr, fun_ptr) =
                    self.order_ctr_fun(symbols, heap, cell_ptr, other_cell_ptr);
                let this = self;
                debug!("Got REDEX bind!!!!");
                scope.spawn(move |scope| this.eval_redex(scope, symbols, heap, ctr_ptr, fun_ptr));
            }
            (_, None) => {
                // value set
            }
        }
    }

    fn eval_connect<'scope>(
        &'scope self,
        scope: &Scope<'scope>,
        symbols: &'scope SymbolBook,
        heap: &'scope Heap<NetF>,
        left_var_ptr: PVarPtr,
        right_var_ptr: PVarPtr,
    ) {
        if self.debug {
            debug!(
                "[{:?}] Evaluating CONNECT: {} ↔ {}",
                rayon::current_thread_index(),
                heap.display_var(symbols, &left_var_ptr.get_fvar_ptr()),
                heap.display_var(symbols, &right_var_ptr.get_fvar_ptr())
            );
        }

        let left_var = heap.get_var(&left_var_ptr);
        let right_var = heap.get_var(&right_var_ptr);

        match (
            left_var.get_store().get_cell_ptr(),
            right_var.get_store().get_cell_ptr(),
        ) {
            (Some(left_cell_ptr), Some(right_cell_ptr)) => {
                let (left_cell_ptr, right_cell_ptr) =
                    self.order_ctr_fun(symbols, heap, left_cell_ptr, right_cell_ptr);

                debug!("Got REDEX!!!");
                scope.spawn(|scope| {
                    self.eval_redex(scope, symbols, heap, left_cell_ptr, right_cell_ptr)
                });
            }
            (None, Some(cell_ptr)) => {
                // Some(Equation::bind(left_var_ptr, cell_ptr))
                debug!(
                    "[{:?}] TODO: wait on left var: {}",
                    thread::current().id(),
                    heap.display_var(symbols, &left_var_ptr.get_fvar_ptr())
                );
                // TODO can we wait on a condition instead??
                scope.spawn(|scope| self.eval_bind(scope, symbols, heap, left_var_ptr, cell_ptr))
            }
            (Some(cell_ptr), None) => {
                // Some(Equation::bind(right_var_ptr, cell_ptr))
                debug!(
                    "[{:?}] TODO: got left var: create bind: {}",
                    thread::current().id(),
                    heap.display_var(symbols, &right_var_ptr.get_fvar_ptr())
                );
                // TODO can we wait on a condition??
                scope.spawn(|scope| self.eval_bind(scope, symbols, heap, right_var_ptr, cell_ptr))
            }
            (None, None) => {
                debug!(
                    "[{:?}] TODO: got nothing: wait for vars {} and {}",
                    thread::current().id(),
                    heap.display_var(symbols, &left_var_ptr.get_fvar_ptr()),
                    heap.display_var(symbols, &right_var_ptr.get_fvar_ptr())
                );
                // TODO can we wait on a condition??
                scope.spawn(|scope| {
                    self.eval_connect(scope, symbols, heap, left_var_ptr, right_var_ptr)
                })
            }
        }
    }

    fn rewrite_equation<'scope>(
        &'scope self,
        scope: &Scope<'scope>,
        symbols: &'scope SymbolBook,
        heap: &'scope Heap<NetF>,
        bvars: &mut PVarPtrBuffer,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_eqn: Equation<RuleF>,
    ) {
        match rule_eqn.get_kind() {
            EquationKind::Redex => self.instantiate_redex(
                scope,
                symbols,
                heap,
                bvars,
                ctr,
                fun,
                &rule_eqn.get_redex_ctr(),
                &rule_eqn.get_redex_fun(),
            ),
            EquationKind::Bind => self.instantiate_bind(
                scope,
                symbols,
                heap,
                bvars,
                ctr,
                fun,
                &rule_eqn.get_bind_var(),
                &rule_eqn.get_bind_cell(),
            ),
            EquationKind::Connect => self.instantiate_connect(
                scope,
                symbols,
                heap,
                bvars,
                ctr,
                fun,
                &rule_eqn.get_connect_left(),
                &rule_eqn.get_connect_right(),
            ),
        }
    }

    fn instantiate_redex<'scope>(
        &'scope self,
        scope: &Scope<'scope>,
        symbols: &'scope SymbolBook,
        heap: &'scope Heap<NetF>,
        bvars: &mut PVarPtrBuffer,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_ctr_ptr: &CellPtr,
        rule_fun_ptr: &CellPtr,
    ) {
        let ctr_ptr = self.instantiate_cell(symbols, heap, bvars, ctr, fun, rule_ctr_ptr);
        let fun_ptr = self.instantiate_cell(symbols, heap, bvars, ctr, fun, rule_fun_ptr);

        if self.debug {
            debug!(
                "[{:?}] Instantiate REDEX: {} = {}  ⟶  {} = {}",
                thread::current().id(),
                self.rules.display_cell(rule_ctr_ptr),
                self.rules.display_cell(rule_fun_ptr),
                heap.display_cell(symbols, &ctr_ptr),
                heap.display_cell(symbols, &fun_ptr)
            );
        }

        scope.spawn(|scope| self.eval_redex(scope, symbols, heap, ctr_ptr, fun_ptr));
    }

    fn instantiate_bind<'scope>(
        &'scope self,
        scope: &Scope<'scope>,
        symbols: &'scope SymbolBook,
        heap: &'scope Heap<NetF>,
        bvars: &mut PVarPtrBuffer,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_var_ptr: &PVarPtr,
        rule_cell_ptr: &CellPtr,
    ) {
        let cell_ptr = self.instantiate_cell(symbols, heap, bvars, ctr, fun, rule_cell_ptr);
        let term_ptr = self.instantiate_var(bvars, ctr, fun, rule_var_ptr);

        match term_ptr.get_kind() {
            TermKind::Cell => {
                if self.debug {
                    debug!(
                        "[{:?}] Instantiate BIND: {} ← {}",
                        thread::current().id(),
                        self.rules.display_var(&rule_var_ptr.get_fvar_ptr()),
                        self.rules.display_cell(rule_cell_ptr)
                    );
                }

                let (ctr_ptr, fun_ptr) =
                    self.order_ctr_fun(symbols, heap, cell_ptr, term_ptr.get_cell_ptr());

                if self.debug {
                    debug!(
                        "  ⟶  {} = {}",
                        heap.display_cell(symbols, &ctr_ptr),
                        heap.display_cell(symbols, &fun_ptr)
                    );
                }

                scope.spawn(|scope| self.eval_redex(scope, symbols, heap, ctr_ptr, fun_ptr));
            }
            TermKind::Var => {
                let pvar_ptr = term_ptr.get_var_ptr();
                let var = heap.get_var(&pvar_ptr);
                match var.get_store().set_or_get(cell_ptr) {
                    (cell_ptr, Some(other_cell_ptr)) => {
                        if self.debug {
                            debug!(
                                "[{:?}] Instantiate BIND: {}[{}] ← {}",
                                thread::current().id(),
                                self.rules.display_var(&rule_var_ptr.get_fvar_ptr()),
                                heap.display_cell(symbols, &other_cell_ptr),
                                self.rules.display_cell(rule_cell_ptr)
                            );
                        }
                        let (ctr_ptr, fun_ptr) =
                            self.order_ctr_fun(symbols, heap, cell_ptr, other_cell_ptr);
                        if self.debug {
                            debug!(
                                "  ⟶  {} = {}",
                                heap.display_cell(symbols, &ctr_ptr),
                                heap.display_cell(symbols, &fun_ptr)
                            );
                        }

                        scope
                            .spawn(|scope| self.eval_redex(scope, symbols, heap, ctr_ptr, fun_ptr));
                    }
                    (cell_ptr, None) => {
                        if self.debug {
                            debug!(
                                "[{:?}] Instantiate BIND: {} ← {}  ⟶  {} ← {}",
                                thread::current().id(),
                                self.rules.display_var(&rule_var_ptr.get_fvar_ptr()),
                                self.rules.display_cell(rule_cell_ptr),
                                heap.display_var(symbols, &term_ptr.get_var_ptr().get_fvar_ptr()),
                                heap.display_cell(symbols, &cell_ptr)
                            );
                        }
                        // value set, do nothing
                    }
                }
            }
        }
    }

    fn instantiate_connect<'scope>(
        &'scope self,
        scope: &Scope<'scope>,
        symbols: &'scope SymbolBook,
        heap: &'scope Heap<NetF>,
        bvars: &mut PVarPtrBuffer,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_left_var: &PVarPtr,
        rule_right_var: &PVarPtr,
    ) {
        let left_port_ptr = self.instantiate_var(bvars, ctr, fun, rule_left_var);
        let right_port_ptr = self.instantiate_var(bvars, ctr, fun, rule_right_var);

        if self.debug {
            debug!(
                "[{:?}] Instantiate CONNECT: {} ↔ {}",
                thread::current().id(),
                self.rules.display_var(&rule_left_var.get_fvar_ptr()),
                self.rules.display_var(&rule_right_var.get_fvar_ptr())
            );
        }

        match (left_port_ptr.get_kind(), right_port_ptr.get_kind()) {
            (TermKind::Cell, TermKind::Cell) => {
                let (ctr_ptr, fun_ptr) = self.order_ctr_fun(
                    symbols,
                    heap,
                    left_port_ptr.get_cell_ptr(),
                    right_port_ptr.get_cell_ptr(),
                );
                if self.debug {
                    debug!(
                        "  ⟶  {} = {}",
                        heap.display_cell(symbols, &ctr_ptr),
                        heap.display_cell(symbols, &fun_ptr)
                    );
                }

                scope.spawn(|scope| self.eval_redex(scope, symbols, heap, ctr_ptr, fun_ptr));
            }
            (TermKind::Cell, TermKind::Var) => {
                // bind
                if self.debug {
                    debug!(
                        "  ⟶  {} ← {}",
                        heap.display_var(symbols, &right_port_ptr.get_var_ptr().get_fvar_ptr()),
                        heap.display_cell(symbols, &left_port_ptr.get_cell_ptr())
                    );
                }

                scope.spawn(move |scope| {
                    self.eval_bind(
                        scope,
                        symbols,
                        heap,
                        right_port_ptr.get_var_ptr().into(),
                        left_port_ptr.get_cell_ptr(),
                    )
                });
            }
            (TermKind::Var, TermKind::Cell) => {
                // bind
                if self.debug {
                    debug!(
                        "  ⟶  {} ← {}",
                        heap.display_var(symbols, &left_port_ptr.get_var_ptr().get_fvar_ptr()),
                        heap.display_cell(symbols, &right_port_ptr.get_cell_ptr())
                    );
                }

                scope.spawn(move |scope| {
                    self.eval_bind(
                        scope,
                        symbols,
                        heap,
                        left_port_ptr.get_var_ptr().into(),
                        right_port_ptr.get_cell_ptr(),
                    )
                });
            }
            (TermKind::Var, TermKind::Var) => {
                // connect
                if self.debug {
                    debug!(
                        "  ⟶  {} ↔ {}",
                        heap.display_var(symbols, &right_port_ptr.get_var_ptr().get_fvar_ptr()),
                        heap.display_var(symbols, &right_port_ptr.get_var_ptr().get_fvar_ptr())
                    );
                }

                scope.spawn(move |scope| {
                    self.eval_connect(
                        scope,
                        symbols,
                        heap,
                        left_port_ptr.get_var_ptr(),
                        right_port_ptr.get_var_ptr(),
                    )
                });
            }
        }
    }

    pub fn instantiate_cell(
        &self,
        symbols: &SymbolBook,
        heap: &Heap<NetF>,
        bvars: &mut PVarPtrBuffer,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_cell_ptr: &CellPtr,
    ) -> CellPtr {
        let rule_cell = self.rules.heap.get_cell(rule_cell_ptr);
        let cell_ptr = match rule_cell.get_symbol_ptr().get_arity() {
            SymbolArity::Zero => heap.cell0(rule_cell.get_symbol_ptr()),
            SymbolArity::One => {
                let term_ptr = self.instantiate_port(
                    symbols,
                    heap,
                    bvars,
                    ctr,
                    fun,
                    rule_cell.get_left_port(),
                );
                let ptr = heap.cell1(rule_cell.get_symbol_ptr(), term_ptr);
                ptr

            }
            SymbolArity::Two => {
                let left_port_ptr = self.instantiate_port(
                    symbols,
                    heap,
                    bvars,
                    ctr,
                    fun,
                    rule_cell.get_left_port(),
                );
                let right_port_ptr = self.instantiate_port(
                    symbols,
                    heap,
                    bvars,
                    ctr,
                    fun,
                    rule_cell.get_right_port(),
                );

                heap.cell2(&rule_cell.get_symbol_ptr(), left_port_ptr, right_port_ptr)
            }
        };
        debug!("[{}] Instantiate CELL: {:?}", rayon::current_thread_index().unwrap(), cell_ptr);

        cell_ptr
    }

    fn instantiate_port(
        &self,
        symbols: &SymbolBook,
        heap: &Heap<NetF>,
        bvars: &mut PVarPtrBuffer,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_port_ptr: TermPtr,
    ) -> TermPtr {
        match rule_port_ptr.get_kind() {
            TermKind::Cell => {
                self.instantiate_cell(
                    symbols,
                    heap,
                    bvars,
                    ctr,
                    fun,
                    &rule_port_ptr.get_cell_ptr(),
                )
                .into()
            }
            TermKind::Var => {
                self.instantiate_var(bvars, ctr, fun, &rule_port_ptr.get_var_ptr())
            }
        }
    }

    fn instantiate_var(
        &self,
        bvars: &mut PVarPtrBuffer,
        ctr: Cell<NetF>,
        fun: Cell<NetF>,
        rule_var_ptr: &PVarPtr,
    ) -> TermPtr {
        let rule_var = self.rules.heap.get_var(rule_var_ptr.into());
        match rule_var {
            Var::Bound(bvar_id) => {
                let bvar = match rule_var_ptr.get_polarity() {
                    Polarity::Pos => bvars.get_pos_var(*bvar_id).into(),
                    Polarity::Neg => bvars.get_neg_var(*bvar_id).into(),
                };
                debug!("[{}] Instantiate Rule BVar {} -> {:?}", rayon::current_thread_index().unwrap(), bvar_id, bvar);
                bvar
            }
            Var::Free(port) => {
                let fvar = self.resolve_fvar(ctr, fun, port);
                debug!("[{}] Instantiate Rule FVar {:?} -> {:?}", rayon::current_thread_index().unwrap(), port, fvar);
                fvar
            }
        }
    }

    fn resolve_fvar(&self, ctr: Cell<NetF>, fun: Cell<NetF>, port: &RulePort) -> TermPtr {
        match port {
            RulePort::Ctr(port_num) => ctr.get_port(*port_num),
            RulePort::Fun(port_num) => fun.get_port(*port_num),
        }
    }

    fn order_ctr_fun(
        &self,
        symbols: &SymbolBook,
        heap: &Heap<NetF>,
        left_ptr: CellPtr,
        right_ptr: CellPtr,
    ) -> (CellPtr, CellPtr) {
        match (left_ptr.get_polarity(), right_ptr.get_polarity()) {
            (Polarity::Pos, Polarity::Neg) => (left_ptr, right_ptr),
            (Polarity::Neg, Polarity::Pos) => (right_ptr, left_ptr),
            (Polarity::Neg, Polarity::Neg) => panic!(
                "Short-circuit (Neg x Neg): {} x {} ({:?} x {:?})",
                heap.display_cell(symbols, &left_ptr),
                heap.display_cell(symbols, &right_ptr),
                left_ptr,
                right_ptr
            ),
            (Polarity::Pos, Polarity::Pos) => panic!(
                "Short-circuit (Pos x Pos): {} x {} ({:?} x {:?})",
                heap.display_cell(symbols, &left_ptr),
                heap.display_cell(symbols, &right_ptr),
                left_ptr,
                right_ptr
            ),
        }
    }

    pub(crate) fn new_bvar_buffer(
        &self,
        symbols: &SymbolBook,
        heap: &Heap<NetF>,
        bvar_count: u8,
    ) -> PVarPtrBuffer {
        let mut buffer = PVarPtrBuffer::new(bvar_count);
        for i in 0..bvar_count {
            buffer.set(i, heap.bvar(NetStore::default()))
        }
        buffer
    }
}

// impl<'a> Runtime<'a> {
//     pub fn display_equation(
//         &self,
//         equation: &'a Equation<NetF>,
//     ) -> EquationDisplay<'a, NetF> {
//         EquationDisplay {
//             equation,
//             symbols,
//             heap: &self.heap,
//         }
//     }

//     pub fn display_cell(&self, cell_ptr: CellPtr) -> CellDisplay<'a, NetF> {
//         self.heap.display_cell(self.symbols, cell_ptr)
//     }
// }

pub struct EquationsBuffer {
    buffer: [Equation<NetF>; Self::MAX_BUFFER_LEN],
    len: u8,
}

impl EquationsBuffer {
    const MAX_BUFFER_LEN: usize = 10;

    pub fn new() -> Self {
        Self {
            len: 0,
            buffer: [Default::default(); Self::MAX_BUFFER_LEN],
        }
    }

    pub fn push(&mut self, eqn: Equation<NetF>) {
        assert!(Self::MAX_BUFFER_LEN > self.len as usize);
        self.buffer[self.len as usize] = eqn;
        self.len += 1
    }

    pub fn get(&self, index: u8) -> Equation<NetF> {
        assert!(index < self.len);
        self.buffer[index as usize]
    }

    pub fn slice(&self) -> &[Equation<NetF>] {
        &self.buffer[0..self.len as usize]
    }
}
