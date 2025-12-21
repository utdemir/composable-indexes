pub use super::Key;

/// Trait of indexes. You probably only need this if you're implementing a new index.
pub trait Index<In> {
    #[doc(hidden)]
    fn insert(&mut self, op: &Insert<In>);

    #[doc(hidden)]
    fn remove(&mut self, op: &Remove<In>);

    #[doc(hidden)]
    #[inline]
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

#[derive(Clone)]
pub struct Insert<'t, In> {
    pub key: Key,
    pub new: &'t In,
}

#[derive(Clone)]
pub struct Update<'t, In> {
    pub key: Key,
    pub new: &'t In,
    pub existing: &'t In,
}

#[derive(Clone)]
pub struct Remove<'t, In> {
    pub key: Key,
    pub existing: &'t In,
}
