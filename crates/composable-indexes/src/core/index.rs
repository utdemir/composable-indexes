pub use super::Key;

/// Seal type to restrict modification of indexes outside
/// of the [`Collection`] API.
///
/// If you are writing tests and need to create a seal,
/// you can use the `unsafe_mk_seal` function under the
/// `testutils` feature flag.
#[derive(Clone, Copy)]
pub struct Seal {
    _seal: (),
}

pub(crate) const SEAL: Seal = Seal { _seal: () };

impl Seal {
    #[cfg(feature = "testutils")]
    pub fn unsafe_mk_seal() -> Self {
        Seal { _seal: () }
    }
}

/// Trait of indexes.
pub trait Index<T> {
    fn insert(&mut self, seal: Seal, op: &Insert<T>);

    fn remove(&mut self, seal: Seal, op: &Remove<T>);

    #[inline]
    fn update(&mut self, seal: Seal, op: &Update<T>) {
        self.remove(
            seal,
            &Remove {
                key: op.key,
                existing: op.existing,
            },
        );
        self.insert(
            seal,
            &Insert {
                key: op.key,
                new: op.new,
            },
        );
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
