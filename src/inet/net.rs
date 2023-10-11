use std::{
    fmt::Display,
    marker::PhantomData,
    sync::atomic::{AtomicU32, Ordering},
};

use tracing::{debug, warn};

use super::{
    cell::CellPtr,
    equation::{Equation, EquationBuilder, EquationsDisplay},
    heap::Heap,
    symbol::SymbolBook,
    term::TermFamily,
    var::{PVarPtr, Var},
};

#[derive(Debug, Copy, Clone)]
pub struct NetF {}
impl TermFamily for NetF {
    type BoundStore = NetVar;
    type FreeStore = NetVar;

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
pub struct NetVar(AtomicU32);

impl NetVar {
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
                // debug!(
                //     "Swapping var value {:?} with {:?}",
                //     cell_ptr,
                //     self.get_cell_ptr()
                // );
                (cell_ptr, Some(CellPtr::from(old_value)))
            } else {
                warn!(
                    "WARN: Setting var with value {:?} twice?",
                    self.get_cell_ptr()
                );
                return (cell_ptr, None);
            }
        } else {
            // debug!("Setting var with value {:?}", self.get_cell_ptr());
            return (cell_ptr, None);
        }
    }
}

impl Default for NetVar {
    fn default() -> Self {
        Self(AtomicU32::new(Self::NULL))
    }
}

impl Var<NetF> {
    pub fn get_store(&self) -> &NetVar {
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
        self.net.head.iter().copied().fold(Ok(()), |result, fvar_ptr| {
            result.and_then(|_| {
                let fvar = self.net.heap.get_var(fvar_ptr);
                assert!(fvar.is_free());
                match fvar {
                    Var::Bound(_) => unreachable!(),
                    Var::Free(store) => match store.get_cell_ptr() {
                        Some(cell_ptr) => write!(
                            f,
                            " _.{}={}",
                            fvar_ptr.get_fvar_ptr().get_index(),
                            self.net.heap.display_cell(self.net.symbols, cell_ptr)
                        ),
                        None => write!(f, " _.{}", fvar_ptr.get_fvar_ptr().get_index()),
                    },
                }
            })
        })
    }
}
