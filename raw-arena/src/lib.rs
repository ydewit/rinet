pub mod arenaraw;

use std::fmt::Debug;

pub trait Ptr: Debug + Clone + Copy {
    fn get_index(&self) -> usize;
}

pub trait ArenaValue<P: Ptr>: Debug {
    fn to_ptr(&self, index: usize) -> P;
}
