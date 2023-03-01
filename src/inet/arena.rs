use std::marker::PhantomData;

pub trait ToPtr<P> {
    fn to_ptr(&self, index: usize) -> P;
}

pub struct ArenaIter<'a, E: ToPtr<P>, P> {
    index: usize,
    entries: &'a Vec<E>,
    _p: PhantomData<P>,
}

impl<'a, E: ToPtr<P>, P> ArenaIter<'a, E, P> {
    pub fn new(entries: &'a Vec<E>) -> Self {
        Self {
            index: 0,
            entries,
            _p: PhantomData,
        }
    }
}

impl<'a, E: ToPtr<P>, P> Iterator for ArenaIter<'a, E, P> {
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.entries.len() {
            let eqn = &self.entries[self.index];
            let ptr = eqn.to_ptr(self.index);
            self.index += 1;
            Some(ptr)
        } else {
            None
        }
    }
}
