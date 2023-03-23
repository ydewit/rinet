use std::fmt::Display;

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
        // print!("alloc {:?}[", &cell0);
        let ptr = self.cells.alloc(cell0);
        // println!("{}]", ptr.get_index());
        ptr
    }

    pub fn cell1(&self, symbol_ptr: SymbolPtr, left_port: TermPtr) -> CellPtr {
        let cell1 = Cell::new1(symbol_ptr, left_port);
        // print!("alloc {:?}[", &cell1);
        let ptr = self.cells.alloc(cell1);
        // println!("{}]", ptr.get_index());
        ptr
    }

    pub fn cell2(
        &self,
        symbol_ptr: &SymbolPtr,
        left_port: TermPtr,
        right_port: TermPtr,
    ) -> CellPtr {
        let cell2 = Cell::new2(symbol_ptr, left_port, right_port);
        // print!("alloc {:?}[", &cell2);
        let ptr = self.cells.alloc(cell2);
        // println!("{}]", ptr.get_index());
        ptr
    }

    pub fn get_cell<'a>(&'a self, cell_ptr: &'a CellPtr) -> &'a Cell<T> {
        self.cells.get(cell_ptr).unwrap()
    }

    // pub fn cells(&self) -> ArenaPtrIter<Cell<T>, CellPtr> {
    //     self.cells.iter()
    // }

    pub fn free_cell(&self, cell_ptr: CellPtr) -> Cell<T> {
        self.cells.free(cell_ptr)
    }

    pub fn bvar(&self, store: T::BoundStore) -> VarPtr {
        self.vars.alloc(Var::Bound(store))
    }

    pub fn fvar(&self, store: T::FreeStore) -> VarPtr {
        self.vars.alloc(Var::Free(store))
    }

    pub fn get_var<'a>(&'a self, var_ptr: &'a PVarPtr) -> &'a Var<T> {
        self.vars.get(&var_ptr.into()).unwrap()
    }

    // pub fn vars(&self) -> ArenaPtrIter<Var<T>, VarPtr> {
    //     self.vars.iter()
    // }

    pub fn free_var(&self, var_ptr: PVarPtr) -> Var<T> {
        self.vars.free(var_ptr.into())
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
        cell_ptr: &'a CellPtr,
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
                .display_cell(self.symbols, &self.term_ptr.get_cell_ptr())
                .fmt(f),
            TermKind::Var => self
                .heap
                .display_var(self.symbols, self.term_ptr.get_var_ptr().into())
                .fmt(f),
        }
    }
}

pub struct CellDisplay<'a, T: TermFamily> {
    cell_ptr: &'a CellPtr,
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
        let var = self.heap.vars.get(&self.var_ptr.into()).unwrap();
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
