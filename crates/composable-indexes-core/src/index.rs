use crate::collection::{Insert, Remove, Update};

/// Trait of indexes. You probably only need this if you're implementing a new index.
pub trait Index<In> {
    #[doc(hidden)]
    fn insert(&mut self, op: &Insert<In>);

    #[doc(hidden)]
    fn remove(&mut self, op: &Remove<In>);

    #[doc(hidden)]
    fn update(&mut self, op: &Update<In>) {
        self.remove(&Remove {
            key: op.key,
            existing: op.existing,
        });
        self.insert(&Insert {
            key: op.key,
            new: op.new,
        });
    }
}
