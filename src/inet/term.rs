use std::fmt::Debug;

pub trait TermFamily: Clone {
    type Store: Debug;

    fn display_store(
        f: &mut std::fmt::Formatter<'_>,
        store: &Self::Store,
        index: usize,
    ) -> std::fmt::Result;
}
