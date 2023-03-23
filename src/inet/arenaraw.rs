use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{alloc::Layout, ptr::NonNull};

use std::{alloc, thread};

use super::arena::{ArenaEntry, ArenaPtr, ArenaValue, SimplePtr};

const FREE_SIZE: usize = 1 << 32;

thread_local! {
    static FREE: RefCell<HashMap<usize, Vec<usize>>> = RefCell::new(Default::default());
}

/// An implementation of Arena that does not use Vec as the underlying storage
/// because we want to allow cross-thread references and mutable references
/// (INets are liner after all so we dont need the compiler to save us from ourselves)
#[derive(Debug)]
pub struct RawArena<T: ArenaValue<P>, P: ArenaPtr = SimplePtr> {
    mem: NonNull<ArenaEntry<T>>, // raw mutable pointer, non-zero, and covariant (?)
    len: AtomicUsize,
    capacity: usize,
    layout: Layout,
    _p: PhantomData<P>,
}

// safe to send to other threads
unsafe impl<T: ArenaValue<P>, P: ArenaPtr> Send for RawArena<T, P> {}
unsafe impl<T: ArenaValue<P>, P: ArenaPtr> Sync for RawArena<T, P> {}

impl<T: ArenaValue<P>, P: ArenaPtr> RawArena<T, P> {
    pub fn new() -> Self {
        Self::with_capacity(FREE_SIZE)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let layout: Layout =
            Layout::array::<ArenaEntry<T>>(capacity).expect("Could not allocate arena");
        let ptr = unsafe { alloc::alloc(layout) } as *mut ArenaEntry<T>;
        Self {
            mem: NonNull::new(ptr).expect("Could not allocate Nonnull"),
            len: AtomicUsize::new(0),
            capacity,
            layout,
            _p: PhantomData,
        }
    }

    fn get_key(&self) -> usize {
        self as *const _ as usize
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    fn push_free_index(&self, index: usize) {
        FREE.with(|f| {
            // println!("New free index: {}", index);
            let mut frees = f.borrow_mut();
            match frees.get_mut(&self.get_key()) {
                Some(free) => free.push(index),
                None => {
                    frees.insert(self.get_key(), vec![index]);
                }
            }
        });
    }

    fn pop_free_index(&self) -> Option<usize> {
        FREE.with(|f| match f.borrow_mut().get_mut(&self.get_key()) {
            Some(free) => free.pop().map(|index| {
                // println!(
                //     "[{:?}] Reusing arena({}) index: {} (len={})",
                //     thread::current().id(),
                //     self.get_key(),
                //     index,
                //     free.len()
                // );
                index
            }),
            None => None,
        })
    }

    pub fn alloc(&self, value: T) -> P {
        let index = match self.pop_free_index() {
            Some(index) => {
                assert!(index < self.len());
                index
            }
            None => {
                let index = self.len.fetch_add(1, Ordering::SeqCst);
                // println!(
                //     "[{:?}] Allocating new arena({}) index: {} (capacity={})",
                //     thread::current().id(),
                //     self.get_key(),
                //     index,
                //     self.capacity()
                // );
                // println!("Len: {}", index + 1);
                assert!(
                    index < self.capacity(),
                    "Max capacity reached: {}",
                    self.capacity()
                );
                index
            }
        };

        let offset = index
            .checked_mul(std::mem::size_of::<ArenaEntry<T>>())
            .expect("Cannot reach memory location");

        assert!(offset < isize::MAX as usize, "Wrapped isize");

        let ptr = value.to_ptr(index);
        // println!("Alloc[{:?}]: {:?}", &ptr, &value);
        let entry = ArenaEntry::Occupied(value);
        unsafe { self.mem.as_ptr().add(index).write(entry) }
        ptr
    }

    pub fn get<'a>(&'a self, ptr: &P) -> Option<&'a T> {
        assert!(
            ptr.get_index() < self.len(),
            "Ptr index is out of bounds {}: {:?}",
            self.len(),
            ptr
        );
        match unsafe { &*self.mem.as_ptr().add(ptr.get_index()) } {
            ArenaEntry::Occupied(value) => Some(value),
            ArenaEntry::Free(_) => panic!("Trying to get a Free arena index: {:?}", ptr),
        }
    }

    pub fn set(&self, ptr: &P, new_value: T) -> T {
        assert!(ptr.get_index() < self.len());
        unsafe {
            let mem_ptr = self.mem.as_ptr().add(ptr.get_index());
            match mem_ptr.read() {
                ArenaEntry::Occupied(value) => {
                    mem_ptr.write(ArenaEntry::Occupied(new_value));
                    value
                }
                ArenaEntry::Free(_) => unreachable!(),
            }
        }
    }

    pub fn free(&self, ptr: P) -> T {
        assert!(ptr.get_index() < self.capacity);
        self.push_free_index(ptr.get_index());
        unsafe {
            let mem_ptr = self.mem.as_ptr().add(ptr.get_index());
            match mem_ptr.read() {
                ArenaEntry::Occupied(value) => {
                    mem_ptr.write(ArenaEntry::Free(ptr.get_index()));
                    value
                }
                ArenaEntry::Free(_) => unreachable!(),
            }
        }
    }
}

impl<T: ArenaValue<P>, P: ArenaPtr> Drop for RawArena<T, P> {
    fn drop(&mut self) {
        unsafe {
            // does not need to bring to the stack to
            std::ptr::drop_in_place(std::slice::from_raw_parts_mut(
                self.mem.as_ptr(),
                self.len(),
            ));
            alloc::dealloc(self.mem.as_ptr() as _, self.layout);
        };
    }
}

impl ArenaValue<SimplePtr> for usize {
    fn to_ptr(&self, index: usize) -> SimplePtr {
        SimplePtr { index }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let a = Vec::new();
        let mut vec = RawArena::<usize>::new();
        assert_eq!(vec.len(), 0);
        let ptr = vec.alloc(6);
        assert_eq!(vec.len(), 1);
        let old = vec.set(&ptr, 11);
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(&ptr), Some(&11));
        assert_eq!(Some(&old), Some(&6));
        assert_eq!(vec.free(ptr), 11);
        assert_eq!(vec.len(), 0);
    }
}

// use std::thread::*;
// thread_local! {
//     static free: RefCell<Vec<usize>> = RefCell::new(Vec::new())
// }

// fn t() {
//     free.with(|f|{

//         f.borrow_mut().push(2);
//     });
// }
