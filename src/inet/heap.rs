use std::fmt::Display;

use tracing::debug;

use super::{
    cell::{Cell, CellPtr, Cells},
    symbol::{SymbolArity, SymbolBook, SymbolPtr},
    term::{TermFamily, TermKind, TermPtr},
    var::{PVarPtr, Var, VarPtr, Vars},
};

#[derive(Debug)]
pub struct Heap<T: TermFamily> {
    pub(crate) cells: Cells<T>,
    pub(crate) vars: Vars<T>,
}

unsafe impl<F: TermFamily> Send for Heap<F> {}
unsafe impl<F: TermFamily> Sync for Heap<F> {}

impl<T: TermFamily> Heap<T> {
    pub fn new() -> Self {
        Self {
            cells: Cells::new(),
            vars: Vars::new(),
        }
    }

    pub fn with_capacity(cells_capacity: usize, vars_capacity: usize) -> Heap<T> {
        Self {
            cells: Cells::with_capacity(cells_capacity),
            vars: Vars::with_capacity(vars_capacity),
        }
    }

    pub fn cell0(&self, symbol_ptr: SymbolPtr) -> CellPtr {
        let cell0 = Cell::new0(symbol_ptr);
        let ptr = self.cells.alloc(cell0.clone());
        tracing::trace!(
            "[Len={}] Alloc CELLS[{}] = {:?}",
            self.cells.len(),
            ptr.get_index(),
            cell0
        );
        ptr
    }

    pub fn reuse_cell0(&self, symbol_ptr: SymbolPtr, ptr: CellPtr) -> CellPtr {
        let cell0 = Cell::new0(symbol_ptr);
        tracing::trace!(
            "[Len={}] Reuse CELLS[{}] = {:?}",
            self.cells.len(),
            ptr.get_index(),
            cell0
        );
        self.cells.alloc_with_ptr(cell0.clone(), ptr)
    }

    pub fn cell1(&self, symbol_ptr: SymbolPtr, left_port: TermPtr) -> CellPtr {
        let cell1 = Cell::new1(symbol_ptr, left_port);
        let ptr = self.cells.alloc(cell1.clone());
        tracing::trace!(
            "[Len={}] Alloc CELLS[{}] = {:?}",
            self.cells.len(),
            ptr.get_index(),
            cell1
        );
        ptr
    }

    pub fn reuse_cell1(&self, symbol_ptr: SymbolPtr, left_port: TermPtr, ptr: CellPtr) -> CellPtr {
        let cell1 = Cell::new1(symbol_ptr, left_port);
        tracing::trace!(
            "[Len={}] Reuse CELLS[{}] = {:?}",
            self.cells.len(),
            ptr.get_index(),
            cell1
        );
        self.cells.alloc_with_ptr(cell1, ptr)
    }

    pub fn cell2(&self, symbol_ptr: SymbolPtr, left_port: TermPtr, right_port: TermPtr) -> CellPtr {
        let cell2 = Cell::new2(symbol_ptr, left_port, right_port);
        let ptr = self.cells.alloc(cell2.clone());
        tracing::trace!(
            "[Len={}] Alloc CELLS[{}] = {:?}",
            self.cells.len(),
            ptr.get_index(),
            cell2
        );
        ptr
    }

    pub fn reuse_cell2(
        &self,
        symbol_ptr: SymbolPtr,
        left_port: TermPtr,
        right_port: TermPtr,
        ptr: CellPtr,
    ) -> CellPtr {
        let cell2 = Cell::new2(symbol_ptr, left_port, right_port);
        tracing::trace!(
            "[Len={}] Reuse CELLS[{}] = {:?}",
            self.cells.len(),
            ptr.get_index(),
            &cell2
        );
        self.cells.alloc_with_ptr(cell2, ptr)
    }

    pub fn get_cell<'a>(&'a self, cell_ptr: CellPtr) -> &'a Cell<T> {
        self.cells.get(cell_ptr).unwrap()
    }

    // pub fn cells(&self) -> ArenaPtrIter<Cell<T>, CellPtr> {
    //     self.cells.iter()
    // }

    pub fn get_var<'a>(&'a self, var_ptr: PVarPtr) -> &'a Var<T> {
        self.vars.get(var_ptr.into()).unwrap()
    }

    pub fn free_cell(&self, cell_ptr: CellPtr) -> Cell<T> {
        let index = cell_ptr.get_index();
        let cell = self.cells.free(cell_ptr);
        tracing::trace!(
            "[Len={}] Free CELLS[{}] = {:?}",
            self.cells.len(),
            index,
            cell
        );
        cell
    }

    pub fn bvar(&self, store: T::BoundStore) -> VarPtr {
        let var_ptr = self.vars.alloc(Var::Bound(store));
        tracing::trace!(
            "[Len={}] Alloc VARS[{}] = BVar",
            self.vars.len(),
            var_ptr.get_index()
        );
        var_ptr
    }

    pub fn fvar(&self, store: T::FreeStore) -> VarPtr {
        let var_ptr = self.vars.alloc(Var::Free(store));
        tracing::trace!(
            "[Len={}] Alloc VARS[{}] = FVar",
            self.vars.len(),
            var_ptr.get_index()
        );
        var_ptr
    }

    // pub fn vars(&self) -> ArenaPtrIter<Var<T>, VarPtr> {
    //     self.vars.iter()
    // }

    pub fn free_var(&self, var_ptr: PVarPtr) -> Var<T> {
        let index = var_ptr.get_fvar_ptr().get_index();
        let var = self.vars.free(var_ptr.into());
        tracing::trace!(
            "[Len={}] Free VARS[{}] -> {:?}",
            self.vars.len(),
            index,
            var
        );
        var
    }

    pub fn display<'a>(&'a self, symbols: &'a SymbolBook) -> HeapDisplay<T> {
        HeapDisplay {
            symbols: symbols,
            heap: &self,
        }
    }

    pub fn display_cell<'a>(
        &'a self,
        symbols: &'a SymbolBook,
        cell_ptr: CellPtr,
    ) -> CellDisplay<T> {
        CellDisplay {
            cell_ptr,
            symbols: symbols,
            heap: &self,
        }
    }

    pub fn display_term<'a>(
        &'a self,
        symbols: &'a SymbolBook,
        term_ptr: TermPtr,
    ) -> TermDisplay<T> {
        TermDisplay {
            term_ptr: term_ptr,
            symbols: symbols,
            heap: &self,
        }
    }

    pub fn display_var<'a>(&'a self, symbols: &'a SymbolBook, var_ptr: VarPtr) -> VarDisplay<T> {
        VarDisplay {
            var_ptr,
            symbols: symbols,
            heap: &self,
        }
    }

    // pub fn display_vars<'a>(&'a self, symbols: &'a SymbolBook) -> VarsDisplay<T> {
    //     VarsDisplay {
    //         symbols: symbols,
    //         heap: &self,
    //     }
    // }
}

pub struct HeapDisplay<'a, T: TermFamily> {
    symbols: &'a SymbolBook,
    heap: &'a Heap<T>,
}
impl<'a, T: TermFamily> Display for HeapDisplay<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Result::Ok(())
    }
}

pub struct TermDisplay<'a, T: TermFamily> {
    term_ptr: TermPtr,
    symbols: &'a SymbolBook,
    heap: &'a Heap<T>,
}
impl<'a, T: TermFamily> Display for TermDisplay<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.term_ptr.get_kind() {
            TermKind::Cell => self
                .heap
                .display_cell(self.symbols, self.term_ptr.get_cell_ptr())
                .fmt(f),
            TermKind::Var => self
                .heap
                .display_var(self.symbols, self.term_ptr.get_var_ptr().into())
                .fmt(f),
        }
    }
}

pub struct CellDisplay<'a, T: TermFamily> {
    cell_ptr: CellPtr,
    symbols: &'a SymbolBook,
    heap: &'a Heap<T>,
}
impl<'a, T: TermFamily> Display for CellDisplay<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cell = match self.heap.cells.get(self.cell_ptr) {
            Some(cell) => cell,
            None => panic!("Cell {:?} not found", self.cell_ptr),
        };

        let name = self.symbols.get_name(cell.get_symbol_ptr()).unwrap();
        match cell.get_symbol_ptr().get_arity() {
            SymbolArity::Zero => {
                write!(f, "{}", name)
            }
            SymbolArity::One => {
                write!(
                    f,
                    "({} {})",
                    name,
                    self.heap.display_term(self.symbols, cell.get_left_port())
                )
            }
            SymbolArity::Two => {
                write!(
                    f,
                    "({} {} {})",
                    name,
                    self.heap.display_term(self.symbols, cell.get_left_port()),
                    self.heap.display_term(self.symbols, cell.get_right_port())
                )
            }
        }
    }
}

pub struct VarDisplay<'a, T: TermFamily> {
    var_ptr: VarPtr,
    symbols: &'a SymbolBook,
    heap: &'a Heap<T>,
}
impl<'a, T: TermFamily> Display for VarDisplay<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let var = self.heap.vars.get(self.var_ptr).unwrap();
        T::display_store(f, self.symbols, self.heap, &var, self.var_ptr.get_index())
    }
}

// pub struct VarsDisplay<'a, T: TermFamily> {
//     symbols: &'a SymbolBook,
//     heap: &'a Heap<T>,
// }
// impl<'a, T: TermFamily> Display for VarsDisplay<'a, T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.heap.vars.iter().fold(Ok(()), |result, ptr| {
//             result.and_then(|_| write!(f, " {}", self.heap.display_var(self.symbols, ptr.into())))
//         })
//     }
// }
