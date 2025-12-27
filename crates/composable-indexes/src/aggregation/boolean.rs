use crate::{
    Index, ShallowClone,
    core::{Insert, Remove, Seal, Update},
};

/// An index that provide `all` & `any` boolean queries.
#[derive(Clone)]
pub struct Boolean {
    true_count: usize,
    false_count: usize,
}

impl ShallowClone for Boolean {}

impl Default for Boolean {
    fn default() -> Self {
        Self::new()
    }
}

impl Boolean {
    pub fn new() -> Self {
        Boolean {
            true_count: 0,
            false_count: 0,
        }
    }

    /// Returns `true` if all indexed values are `true` (or if the collection is empty).
    ///
    /// An empty collection returns `true` (vacuous truth).
    #[inline]
    pub fn all(&self) -> bool {
        self.false_count == 0
    }

    /// Returns `true` if at least one indexed value is `true`.
    ///
    /// An empty collection returns `false`.
    #[inline]
    pub fn any(&self) -> bool {
        self.true_count > 0
    }

    /// Returns the count of `true` values.
    #[inline]
    pub fn true_count(&self) -> usize {
        self.true_count
    }

    /// Returns the count of `false` values.
    #[inline]
    pub fn false_count(&self) -> usize {
        self.false_count
    }

    /// Returns the total count of all values (both `true` and `false`).
    #[inline]
    pub fn total_count(&self) -> usize {
        self.true_count + self.false_count
    }
}

impl Index<bool> for Boolean {
    #[inline]
    fn insert(&mut self, _seal: Seal, op: &Insert<bool>) {
        if *op.new {
            self.true_count += 1;
        } else {
            self.false_count += 1;
        }
    }

    #[inline]
    fn remove(&mut self, _seal: Seal, op: &Remove<bool>) {
        if *op.existing {
            self.true_count -= 1;
        } else {
            self.false_count -= 1;
        }
    }

    #[inline]
    fn update(&mut self, _seal: Seal, op: &Update<bool>) {
        match (op.existing, op.new) {
            (false, true) => {
                self.false_count -= 1;
                self.true_count += 1;
            }
            (true, false) => {
                self.true_count -= 1;
                self.false_count += 1;
            }
            _ => {} // No change if both true or both false
        }
    }
}
