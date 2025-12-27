use crate::{
    Index, ShallowClone,
    core::{Insert, Remove, Seal, Update},
};

/// An index that only counts the number of items.
#[derive(Clone)]
pub struct Count {
    count: usize,
}

impl Default for Count {
    fn default() -> Self {
        Self::new()
    }
}

impl Count {
    pub fn new() -> Self {
        Count { count: 0 }
    }
}

impl<_K> Index<_K> for Count {
    #[inline]
    fn insert(&mut self, _seal: Seal, _op: &Insert<_K>) {
        self.count += 1;
    }

    #[inline]
    fn remove(&mut self, _seal: Seal, _op: &Remove<_K>) {
        self.count -= 1;
    }

    #[inline]
    fn update(&mut self, _seal: Seal, _op: &Update<_K>) {
        // No change in count on update
    }
}

impl Count {
    #[inline]
    pub fn count(&self) -> usize {
        self.count
    }
}

impl ShallowClone for Count {}
