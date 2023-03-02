use std::marker::PhantomData;

pub trait ArenaPtr {
    fn get_index(&self) -> usize;
}

pub struct SimplePtr {
    index: usize,
}
impl ArenaPtr for SimplePtr {
    fn get_index(&self) -> usize {
        self.index
    }
}

pub trait ArenaValue<P: ArenaPtr> {
    fn to_ptr(&self, index: usize) -> P;
}

pub struct ArenaPtrIter<'a, T: ArenaValue<P>, P: ArenaPtr> {
    index: usize,
    arena: &'a Arena<T, P>,
}

impl<'a, T: ArenaValue<P>, P: ArenaPtr> ArenaPtrIter<'a, T, P> {
    fn new(arena: &'a Arena<T, P>) -> Self {
        Self { index: 0, arena }
    }
}

impl<'a, T: ArenaValue<P>, P: ArenaPtr> Iterator for ArenaPtrIter<'a, T, P> {
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        for i in self.index..self.arena.entries.len() {
            match &self.arena.entries[i] {
                ArenaEntry::Occupied(value) => {
                    let ptr = value.to_ptr(self.index);
                    self.index += 1;
                    return Some(ptr);
                }
                ArenaEntry::Free(_) => (),
            };
        }
        None
    }
}

pub struct ArenaValueIter<'a, T: ArenaValue<P>, P: ArenaPtr> {
    iter: ArenaPtrIter<'a, T, P>,
}

impl<'a, T: ArenaValue<P>, P: ArenaPtr> ArenaValueIter<'a, T, P> {
    fn new(iter: ArenaPtrIter<'a, T, P>) -> Self {
        Self { iter }
    }
}

impl<'a, T: ArenaValue<P>, P: ArenaPtr> Iterator for ArenaValueIter<'a, T, P> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.iter.next().unwrap();
        self.iter.arena.get(ptr)
    }
}

#[derive(Debug)]
enum ArenaEntry<T> {
    Occupied(T),
    Free(usize),
}

#[derive(Debug)]
pub struct Arena<T: ArenaValue<P>, P: ArenaPtr> {
    entries: Vec<ArenaEntry<T>>,
    free: Vec<usize>,
    _p: PhantomData<P>,
}

impl<T: ArenaValue<P>, P: ArenaPtr> Arena<T, P> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free: Vec::new(),
            _p: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            free: Vec::new(),
            _p: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len() - self.free.len()
    }

    pub fn iter(&self) -> ArenaPtrIter<T, P> {
        ArenaPtrIter::new(&self)
    }

    pub fn values_iter(&self) -> ArenaValueIter<T, P> {
        ArenaValueIter::new(self.iter())
    }

    pub fn drain_values<'a>(&'a mut self) -> impl 'a + Iterator<Item = T> {
        self.entries
            .iter_mut()
            .enumerate()
            .filter_map(|(i, entry)| match entry {
                ArenaEntry::Occupied(_) => {
                    let mut swap_value = ArenaEntry::Free(self.free.len());
                    std::mem::swap(&mut swap_value, entry);
                    self.free.push(i);
                    match swap_value {
                        ArenaEntry::Occupied(v) => Some(v),
                        _ => unreachable!("We already checked that this was occupied"),
                    }
                }
                ArenaEntry::Free(_) => None,
            })
    }

    pub fn get<'a>(&'a self, ptr: P) -> Option<&'a T> {
        if ptr.get_index() >= self.entries.len() {
            return None;
        }

        match &self.entries[ptr.get_index()] {
            ArenaEntry::Occupied(value) => return Some(&value),
            ArenaEntry::Free(_) => None,
        }
    }

    pub fn update(&mut self, ptr: P, new_value: T) -> Option<T> {
        if ptr.get_index() >= self.entries.len() {
            return None;
        }

        match &mut self.entries[ptr.get_index()] {
            ArenaEntry::Occupied(value) => {
                let value = std::mem::replace(value, new_value);
                return Some(value);
            }
            ArenaEntry::Free(_) => None,
        }
    }

    pub fn alloc(&mut self, value: T) -> P {
        let index = self.entries.len();
        let ptr = value.to_ptr(index);
        self.entries.push(ArenaEntry::Occupied(value));
        ptr
    }

    // pub fn extend(&mut self, arena: Arena<T,P>) {
    //     self.entries.extend(arena.iter())
    // }

    pub fn free(&mut self, ptr: P) -> Option<T> {
        if ptr.get_index() >= self.entries.len() {
            return None;
        }

        match &mut self.entries[ptr.get_index()] {
            entry @ ArenaEntry::Occupied(_) => {
                let free_index = self.free.len();
                self.free.push(ptr.get_index());
                let entry = std::mem::replace(entry, ArenaEntry::Free(free_index));
                if let ArenaEntry::Occupied(value) = entry {
                    return Some(value);
                } else {
                    return None;
                }
            }
            ArenaEntry::Free(_) => return None,
        }
    }
}
