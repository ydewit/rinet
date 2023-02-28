

pub enum SmallVector<T, const N: usize> {
    Inline(usize, [T; N]),
    Dynamic(Vec<T>),
}

impl<T: Copy + Clone, const N: usize> SmallVector<T, N> {
    pub fn new(v: T, n: usize) -> Self {
        if n <= N {
            Self::Inline(n, [v; N])
        } else {
            Self::Dynamic(vec![v; n])
        }
    }
}

impl<T, const N: usize> SmallVector<T, N> {
    pub fn as_slice(&self) -> &[T] {
        match self {
            Self::Inline(n, array) => &array[0..*n],
            Self::Dynamic(vec) => vec,
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self {
            Self::Inline(n, array) => &mut array[0..*n],
            Self::Dynamic(vec) => vec,
        }
    }
}

use std::ops::{Deref, DerefMut};

impl<T, const N: usize> Deref for SmallVector<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for SmallVector<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}