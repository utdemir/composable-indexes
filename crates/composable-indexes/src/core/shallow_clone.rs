/**
 * A marker trait for types that support cloning without having to copy all indexed values.
 * This likely means that either:
 *
 * - The type uses persistent data structures internally (e.g., `crate::index::im::BTreeIndex`)
 * - The type only works using aggregations that do not store all samples (e.g., `crate::aggregation::CountIndex`)
 */
pub trait ShallowClone: Clone {
    fn shallow_clone(&self) -> Self {
        self.clone()
    }
}
