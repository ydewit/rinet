// //! [![](https://img.shields.io/crates/v/id-arena.svg)](https://crates.io/crates/id-arena)
// //! [![](https://img.shields.io/crates/d/id-arena.svg)](https://crates.io/crates/id-arena)
// //! [![Travis CI Build Status](https://travis-ci.org/fitzgen/id-arena.svg?branch=master)](https://travis-ci.org/fitzgen/id-arena)

//! A safe arena allocator that allows deletion without suffering from [the ABA
//! problem](https://en.wikipedia.org/wiki/ABA_problem) by using generational indices.
//!
//! # Id-based
//!
//! Allocate value and get back an identifier for that object, and not
//! a reference so that we can use it to construct mutable graphs.
//!
//! # Typed
//!
//!
//! # Homogeneus
//!
//! An arena can contain only one type of object.
//!
//! # Custom Id data
//!
//! Id typing allows additional data associated with the Id. For instance, an Id that is really an enum
//! of two other ids: this is useful in cases you need to model a reference that can take two types of
//! objects (i.e. stored in different )
//!
//! # Droppable?
//!
//!
//! # Example

use std::marker::PhantomData;

const DEFAULT_ENTRY_CAPACITY: usize = 100;
const DEFAULT_FREE_CAPACITY: usize = 20;

/// A typed identifier for an arena allocation
#[derive(Debug, PartialEq, Eq)]
struct ArenaId<T> {
    index: usize,
    generation: u64,
    _t: PhantomData<T>,
}

impl<T> ArenaId<T> {
    fn next_generation(&mut self) -> &mut Self {
        self.generation += 1;
        self
    }
}

impl<T> Clone for ArenaId<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            generation: self.generation,
            _t: PhantomData,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.index = source.index;
        self.generation = source.generation;
        self._t = source._t;
    }
}

impl<T> Copy for ArenaId<T> {}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ValueEntry<T> {
    Occupied { value: T, generation: u64 },
    Free { free_index: usize },
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct Arena<T> {
    entries: Vec<ValueEntry<T>>,
    free: Vec<ArenaId<T>>,
}

impl<T> Arena<T> {
    #[inline]
    pub fn new() -> Arena<T> {
        Arena::with_capacity(DEFAULT_ENTRY_CAPACITY, DEFAULT_FREE_CAPACITY)
    }

    #[inline]
    pub fn with_capacity(entries_capacity: usize, free_capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(entries_capacity),
            free: Vec::with_capacity(free_capacity),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len() - self.free.len()
    }

    #[inline]
    pub fn alloc(&mut self, value: T) -> ArenaId<T> {
        let index = self.entries.len();
        self.entries.push(ValueEntry::Occupied {
            value: value,
            generation: 0,
        });
        Id::new(index, 0)
    }

    #[inline]
    pub fn alloc_with<C: FnOnce() -> T>(&mut self, create_fn: C) -> ArenaId<T> {
        let index = self.entries.len();
        self.entries.push(ValueEntry::Occupied {
            value: create_fn(),
            generation: 0,
        });
        Id::new(index, 0)
    }

    #[inline]
    pub fn get(&self, id: &ArenaId<T>) -> Option<&T> {
        match self.entries.get(id.index) {
            Some(entry) => match entry {
                ValueEntry::Occupied { value, generation } => Some(value),
                ValueEntry::Free { free_index } => None,
            },
            None => None,
        }
    }

    pub fn remove(&mut self, id: &ArenaId<T>) -> Option<T> {
        if id.index >= self.entries.len() {
            return None;
        }

        let entry = &self.entries[id.index];
        if let ValueEntry::Free { free_index } = entry {
            return None;
        }

        let free_index = self.free.len();
        let old_entry =
            std::mem::replace(&mut self.entries[id.index], ValueEntry::Free { free_index });
        match old_entry {
            ValueEntry::Occupied { value, .. } => Some(value),
            ValueEntry::Free { .. } => None,
        }
    }
}
