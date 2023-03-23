use std::{
    fmt::Display,
    marker::PhantomData,
    sync::atomic::{AtomicU32, Ordering},
};

use super::{
    arena::ArenaPtrIter,
    cell::{Cell, CellPtr},
    equation::{
        Equation, EquationBuilder, EquationDisplay, EquationPtr, Equations, EquationsDisplay,
    },
    heap::{CellDisplay, Heap},
    symbol::SymbolBook,
    term::TermFamily,
    var::{PVarPtr, Var, VarPtr},
};

#[derive(Debug, Copy, Clone)]
pub struct NetF {}
impl TermFamily for NetF {
    type BoundStore = NetStore;
    type FreeStore = NetStore;

    fn display_store(
        f: &mut std::fmt::Formatter<'_>,
        symbols: &SymbolBook,
        heap: &Heap<Self>,
        var: &Var<NetF>,
        index: usize,
    ) -> std::fmt::Result {
        match var {
            Var::Bound(store) => match store.get_cell_ptr() {
                // Some(cell_ptr) => write!(f, "x.{}[={}]", index, heap.display_cell(symbols, cell_ptr)),
                // Some(cell_ptr) => write!(f, "{}", heap.display_cell(symbols, cell_ptr)),
                Some(cell_ptr) => write!(f, "x.{}", index),
                None => write!(f, "x.{}", index),
            },
            Var::Free(_) => write!(f, "_.{}", index),
        }
    }
}

#[derive(Debug)]
pub struct NetStore(AtomicU32);

impl NetStore {
    const NULL: u32 = u32::MAX;

    pub fn get_cell_ptr(&self) -> Option<CellPtr> {
        let value = self.0.load(Ordering::SeqCst);
        if value != Self::NULL {
            Some(CellPtr::from(value))
        } else {
            return None;
        }
    }

    pub fn set_or_get(&self, cell_ptr: CellPtr) -> (CellPtr, Option<CellPtr>) {
        let old_value = self.0.swap(cell_ptr.get_ptr(), Ordering::SeqCst);
        if old_value != Self::NULL {
            if old_value != cell_ptr.get_ptr() {
                // println!("Swapping var value {:?} with {:?}", old_cell_ptr, self.get_cell_ptr());
                (cell_ptr, Some(CellPtr::from(old_value)))
            } else {
                println!(
                    "WARN: Setting var with value {:?} twice?",
                    self.get_cell_ptr()
                );
                return (cell_ptr, None);
            }
        } else {
            // println!("Setting var with value {:?}", self.get_cell_ptr());
            return (cell_ptr, None);
        }
    }
}

impl Default for NetStore {
    fn default() -> Self {
        Self(AtomicU32::new(Self::NULL))
    }
}

impl Var<NetF> {
    pub fn get_store(&self) -> &NetStore {
        match self {
            Var::Bound(store) => store,
            Var::Free(store) => store,
        }
    }
}

impl Default for Equation<NetF> {
    fn default() -> Self {
        Self(0, PhantomData)
    }
}

#[derive(Debug)]
pub struct Net<'a> {
    pub symbols: &'a SymbolBook,
    pub head: Vec<PVarPtr>,
    pub body: Vec<Equation<NetF>>,
    pub heap: Heap<NetF>,
}
impl<'a> Net<'a> {
    pub fn new(symbols: &'a SymbolBook) -> Self {
        Self {
            symbols,
            head: Vec::new(),
            body: Vec::new(),
            heap: Heap::new(),
        }
    }

    pub fn with_capacity(symbols: &'a SymbolBook, capacity: [usize; 3]) -> Self {
        Self {
            symbols,
            head: Vec::new(),
            body: Vec::with_capacity(capacity[0]),
            heap: Heap::with_capacity(capacity[1], capacity[2]),
        }
    }

    // Equations --------------------------

    pub fn equations<F>(&mut self, builder_fn: F)
    where
        F: FnOnce(&mut EquationBuilder<NetF>),
    {
        let mut builder = EquationBuilder::new(
            &self.symbols,
            &mut self.head,
            &mut self.body,
            &mut self.heap,
        );
        builder_fn(&mut builder);
        builder.build();
    }

    // pub fn redex(&mut self, ctr_ptr: CellPtr, fun_ptr: CellPtr) {
    //     self.body.push(Equation::redex(ctr_ptr, fun_ptr))
    // }

    // pub fn bind(&mut self, var_ptr: PVarPtr, fun_ptr: CellPtr) {
    //     self.body.push(Equation::bind(var_ptr, fun_ptr))
    // }

    // pub fn connect(&mut self, left_ptr: PVarPtr, right_ptr: PVarPtr) {
    //     self.body.push(Equation::connect(left_ptr, right_ptr))
    // }

    // pub fn body(&self) -> std::slice::Iter<Equation<NetF>>  {
    //     self.body.iter()
    // }

    // pub fn get_body(&'a self, equation: EquationPtr) -> &'a Equation<NetF> {
    //     self.body.get(equation).unwrap()
    // }

    // Cells --------------------------

    // pub fn cells(&self) -> ArenaPtrIter<Cell<NetF>, CellPtr> {
    //     self.heap.cells()
    // }

    // pub fn get_cell(&'a self, cell: &'a CellPtr) -> &'a Cell<NetF> {
    //     self.heap.get_cell(cell)
    // }

    // pub fn free_cell(&mut self, cell_ptr: CellPtr) -> Cell<NetF> {
    //     self.heap.free_cell(cell_ptr)
    // }

    // Vars --------------------------

    // pub fn vars(&self) -> ArenaPtrIter<Var<NetF>, VarPtr> {
    //     self.heap.vars()
    // }

    // pub fn get_vars(&'a self, ptr: &'a PVarPtr) -> &'a Var<NetF> {
    //     self.heap.get_var(ptr.into())
    // }

    // pub fn input(&mut self) -> PVarPtr {
    //     let fvar_ptr = self.heap.fvar(NetStore::default());
    //     let (input_ptr, output_ptr) = PVarPtr::wire(fvar_ptr);
    //     self.head.push(output_ptr);
    //     input_ptr
    // }

    // pub fn output(&mut self) -> PVarPtr {
    //     let fvar_ptr = self.heap.fvar(NetStore::default());
    //     let (input_ptr, output_ptr) = PVarPtr::wire(fvar_ptr);
    //     self.head.push(input_ptr);
    //     output_ptr
    // }

    // pub fn wire(&mut self) -> (PVarPtr, PVarPtr) {
    //     let bvar_ptr = self.heap.bvar(NetStore::default());
    //     PVarPtr::wire(self.bvar())
    // }

    // pub fn bvar(&mut self) -> VarPtr {
    //     self.heap.bvar(NetStore::default())
    // }

    pub fn display_head(&'a self) -> HeadDisplay {
        HeadDisplay { net: self }
    }

    pub fn display_body(&'a self) -> EquationsDisplay<'a, NetF> {
        EquationsDisplay {
            symbols: self.symbols,
            body: &self.body,
            heap: &self.heap,
        }
    }

    // pub fn display_equation(
    //     &'a self,
    //     symbols: &'a SymbolBook,
    //     equation: &'a Equation<NetF>,
    // ) -> EquationDisplay<'a, NetF> {
    //     EquationDisplay {
    //         equation,
    //         symbols,
    //         heap: &self.heap,
    //     }
    // }

    // pub fn display_cell(&'a self, cell_ptr: &'a CellPtr) -> CellDisplay<'a, NetF> {
    //     self.heap.display_cell(self.symbols, cell_ptr)
    // }
}

impl<'a> Display for Net<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{} | {} > ({} cells, {} vars)",
            self.display_head(),
            self.display_body(),
            self.heap.cells.len(),
            self.heap.vars.len()
        )
    }
}

pub struct HeadDisplay<'a> {
    net: &'a Net<'a>,
}
impl<'a> Display for HeadDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.net.head.iter().fold(Ok(()), |result, fvar_ptr| {
            result.and_then(|_| {
                let fvar = self.net.heap.get_var(&fvar_ptr);
                assert!(fvar.is_free());
                match fvar {
                    Var::Bound(_) => unreachable!(),
                    Var::Free(store) => match store.get_cell_ptr() {
                        Some(cell_ptr) => write!(
                            f,
                            " _.{}={}",
                            fvar_ptr.get_fvar_ptr().get_index(),
                            self.net.heap.display_cell(self.net.symbols, &cell_ptr)
                        ),
                        None => write!(f, " _.{}", fvar_ptr.get_fvar_ptr().get_index()),
                    },
                }
            })
        })
    }
}
