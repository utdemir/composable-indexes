pub use super::Key;

/// Seal type to restrict modification of indexes outside
/// of the [`crate::Collection`] API.
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

/// Types that can be used as indexes in a [`crate::Collection`].
///
/// An index processes items of type `T` - and can be accessed through the
/// [`crate::Collection::query`]-family of methods to expose queries over the
/// indexed data.
///
/// You should only need to know about this trait if you are implementing a
/// custom index.
///
/// # Implementing an Index
///
/// An [`Index`] receives insertions, removals, and updates of items of type `T`,
/// and maintains some internal state to answer queries about the indexed data.
///
/// Indexes must (minimally) handle insertion and removal of items. (Update
/// operation has a default implementation that calls remove followed by insert;
/// override it if a more efficient implementation is possible.)
///
/// All trait functions receive a `Seal` parameter that prevents external code
/// from calling them directly. You can safely ignore this parameter.
///
/// All operations receive an operation struct (`Insert`, `Remove`, or `Update`)
/// [`Insert`] guarantees that the key did not previously exists, [`Remove`] and
/// [`Update`] guarantee carry a pointer to the existing value. This usually
/// allows for more efficient implementations.
///
/// The queries themselves are not part of this trait - they are implemented
/// as inherent on the index type itself.
pub trait Index<T> {
    fn insert(&mut self, seal: Seal, op: &Insert<T>);

    fn remove(&mut self, seal: Seal, op: &Remove<T>);

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

    fn vacuum(&mut self, _seal: Seal) {}
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
