use core::slice;
use std::{marker::PhantomData, hash::{Hash, Hasher}, ops::{Index}, iter::{Enumerate, self}, vec::{self}};

use rayon::prelude::IntoParallelRefIterator;

trait ArenaPtr<T> {
    fn new(index: usize) -> Self;
    fn get_index(self) -> usize;
}


#[derive(Debug,Clone,Copy)]
pub struct Ptr<T: Copy> {
    index: usize,
    pub tag: T,
}

impl<T: Copy> Eq for Ptr<T> {}

impl<T: Copy> PartialEq for Ptr<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T: Copy> Hash for Ptr<T> {
    #[inline]
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.index.hash(h);
    }
}

pub trait ToTag<T: Copy> {
    fn to_tag(&self) -> T;
}

#[derive(Debug)]
enum ArenaEntry<E> {
    Occupied(E),
    Free(usize)
}

impl<T> ArenaEntry<T> {
    fn is_occupied(&self) -> bool {
        match self {
            ArenaEntry::Occupied(_) => true,
            ArenaEntry::Free(_) => false,
        }
    }
}


#[derive(Debug, Default)]
pub struct Arena<E: ToTag<T>, T: Copy> {
    entries: Vec<ArenaEntry<E>>,
    free: Vec<usize>,
    _t: PhantomData<T>
}

impl<E: ToTag<T>, T: Copy> Arena<E, T> {
    #[inline]
    pub fn new() -> Self {
        Self { entries: Vec::new(), free: Vec::new(), _t: PhantomData }
    }

    #[inline]
    pub fn with_capacity(entries_capacity: usize, free_capacity: usize) -> Self {
        Self { entries: Vec::with_capacity(entries_capacity), free: Vec::with_capacity(free_capacity), _t: PhantomData }
    }

    /// Get the number of entries in this arena
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len() - self.free.len()
    }

    ///
    #[inline]
    pub fn get(&self, ptr: &Ptr<T>) -> Option<&E> {
        match self.entries.get(ptr.index) {
            Some(entry) => {
                match entry {
                    ArenaEntry::Occupied(e) => Some(e),
                    ArenaEntry::Free(_) => None,
                }
            },
            None => None,
        }
    }

    #[inline]
    pub fn alloc(&mut self, entry: E, tag: T) -> Ptr<T> {
        let entry = ArenaEntry::Occupied(entry);
        if let Some(index) = self.free.pop() {
            self.entries[index] = entry;
            Ptr { index, tag }
        }
        else {
            self.entries.push(entry);
            Ptr { index: self.entries.len(), tag }
        }
    }

    #[inline]
    pub fn free(&mut self, id: Ptr<T>) -> Option<E> {
        match std::mem::replace(&mut self.entries[id.index], ArenaEntry::Free(self.free.len())) {
            ArenaEntry::Occupied(entry) => {
                self.free.push(id.index);
                Some(entry)
            },
            ArenaEntry::Free(_) => None,
        }
    }

    #[inline]
    pub fn iter<'a>(&'a self) -> ArenaIter<E, T> {
        IntoIterator::into_iter(self)
    }
}


impl<E: ToTag<T>, T: Copy> Index<Ptr<T>> for Arena<E,T> {
    type Output = E;

    #[inline]
    fn index(&self, index: Ptr<T>) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

pub struct ArenaIter<'a, E: ToTag<T>, T: Copy> {
    iter: iter::Enumerate<slice::Iter<'a, ArenaEntry<E>>>,
    _e: PhantomData<T>
}

impl<'a, E: ToTag<T>, T: Copy> Iterator for ArenaIter<'a, E, T> {
    type Item = Ptr<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some((index, ArenaEntry::Occupied(entry))) => return Some(Ptr { index, tag: entry.to_tag() }),
                Some((index, ArenaEntry::Free(..))) => continue,
                None => return None,
            }
        }
    }
}

impl<'a, E: ToTag<T>, T: Copy> IntoIterator for &'a Arena<E, T> {
    type Item = Ptr<T>;
    type IntoIter = ArenaIter<'a, E, T>;

    #[inline]
    fn into_iter(self) -> ArenaIter<'a, E, T> {
        ArenaIter {
            iter: self.entries.iter().enumerate(),
            _e: PhantomData,
        }
    }
}
