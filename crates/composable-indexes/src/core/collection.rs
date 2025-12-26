use crate::{
    ShallowClone,
    core::{DefaultStore, SEAL, store::Store},
};

use super::{
    QueryResult, QueryResultDistinct,
    index::{Index, Insert, Remove, Update},
};

#[cfg(test)]
use super::index::Seal;

/// Unique identifier for an item in a collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key {
    id: u64,
}

impl Key {
    pub fn unsafe_from_u64(id: u64) -> Self {
        Key { id }
    }

    pub fn as_u64(&self) -> u64 {
        self.id
    }
}

/// A collection of items, with an index that is automatically kept up-to-date.
#[derive(Clone)]
pub struct Collection<In, Ix, S = DefaultStore<In>> {
    index: Ix,
    data: S,
    next_key_id: u64,
    _marker: core::marker::PhantomData<fn() -> In>,
}

impl<In, Ix> Collection<In, Ix> {
    /// Create an empty collection.
    pub fn new(ix: Ix) -> Self
    where
        In: 'static,
        Ix: Index<In>,
        DefaultStore<In>: Store<In>,
    {
        Collection::new_with_empty_store(ix)
    }
}

impl<In, Ix, S> ShallowClone for Collection<In, Ix, S>
where
    In: Clone,
    Ix: ShallowClone,
    S: ShallowClone,
{
}

impl<In, Ix, S: Default> Collection<In, Ix, S> {
    /// Create an empty collection.
    pub fn new_with_empty_store(ix: Ix) -> Self {
        Collection::new_with_store(S::default(), ix)
    }
}

impl<In, Ix, S> Collection<In, Ix, S> {
    /// Create an empty collection with a custom store.
    pub fn new_with_store(store: S, ix: Ix) -> Self {
        Collection {
            data: store,
            next_key_id: 0,
            index: ix,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<In, Ix, S> Collection<In, Ix, S>
where
    In: 'static,
    Ix: Index<In>,
    S: Store<In>,
{
    /// Lookup an item in the collection by its key.
    pub fn get_by_key(&self, key: Key) -> Option<&In> {
        self.data.get(key)
    }

    /// Insert a new item into the collection.
    pub fn insert(&mut self, value: In) -> Key {
        let key = self.mk_key();
        let existing = self.data.insert(key, value);

        // There shouldn't be an existing key, as we just generated it
        debug_assert!(existing.is_none());

        self.index.insert(
            SEAL,
            &Insert {
                key,
                new: self.data.get_unwrapped(key),
            },
        );

        key
    }

    /// Insert all items from an iterator into the collection.
    pub fn insert_all<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = In>,
    {
        for item in iter {
            self.insert(item);
        }
    }

    /// Iterate over all items in the collection.
    pub fn iter(&self) -> impl IntoIterator<Item = (Key, &In)> {
        self.data.iter()
    }

    /// Mutate (or alter the presence of) the item in the collection.
    pub fn update_by_key_mut<F>(&mut self, key: Key, f: F)
    where
        F: FnOnce(&mut Option<In>),
    {
        let mut existing = self.delete_by_key(key);
        f(&mut existing);

        if let Some(existing) = existing {
            self.data.insert(key, existing);
            self.index.insert(
                SEAL,
                &Insert {
                    key,
                    new: self.data.get_unwrapped(key),
                },
            );
        }
    }

    /// Update the item in the collection.
    pub fn update_by_key<F>(&mut self, key: Key, f: F)
    where
        F: FnOnce(Option<&In>) -> In,
    {
        let existing = self.data.get(key);
        let new = f(existing);

        match existing {
            Some(existing) => {
                self.index.update(
                    SEAL,
                    &Update {
                        key,
                        new: &new,
                        existing,
                    },
                );
                self.data.insert(key, new);
            }
            None => {
                self.index.insert(SEAL, &Insert { key, new: &new });
                self.data.insert(key, new);
            }
        };
    }

    /// Mutate the item in the collection, if it exists.
    pub fn adjust_by_key_mut<F>(&mut self, key: Key, f: F)
    where
        F: FnOnce(&mut In),
    {
        if let Some(mut existing) = self.delete_by_key(key) {
            f(&mut existing);
            self.data.insert(key, existing);
            self.index.insert(
                SEAL,
                &Insert {
                    key,
                    new: self.data.get_unwrapped(key),
                },
            );
        }
    }

    /// Adjust the item in the collection, if it exists.
    pub fn adjust_by_key<F>(&mut self, key: Key, f: F)
    where
        F: FnOnce(&In) -> In,
    {
        if let Some(existing) = self.data.get(key) {
            let new = f(existing);
            self.index.update(
                SEAL,
                &Update {
                    key,
                    new: &new,
                    existing,
                },
            );
            self.data.insert(key, new);
        }
    }

    /// Remove an item from the collection, returning it if it exists.
    pub fn delete_by_key(&mut self, key: Key) -> Option<In> {
        let existing = self.data.remove(key);

        if let Some(ref existing) = existing {
            self.index.remove(SEAL, &Remove { key, existing });
        }

        existing
    }

    /// Query the collection using its index(es).
    pub fn query<Res>(&self, f: impl FnOnce(&Ix) -> Res) -> Res::Resolved<&In>
    where
        Res: QueryResult,
    {
        let res = f(&self.index);
        res.map(|k| self.data.get_unwrapped(k))
    }

    pub fn query_keys<Res>(&self, f: impl FnOnce(&Ix) -> Res) -> Res::Resolved<Key>
    where
        Res: QueryResult,
    {
        let res = f(&self.index);
        res.map(|k| k)
    }

    pub fn query_with_keys<Res>(&self, f: impl FnOnce(&Ix) -> Res) -> Res::Resolved<(Key, &In)>
    where
        Res: QueryResult,
    {
        let res = f(&self.index);
        res.map(|k| (k, self.data.get_unwrapped(k)))
    }

    pub fn delete<Res>(&mut self, f: impl FnOnce(&Ix) -> Res) -> usize
    where
        Res: QueryResult,
    {
        let mut affected_count = 0;
        let res = f(&self.index);
        res.map(|key| {
            self.delete_by_key(key);
            affected_count += 1;
        });
        affected_count
    }

    pub fn update<Res, F>(
        &mut self,
        f: impl FnOnce(&Ix) -> Res,
        update_fn: impl Fn(&In) -> In,
    ) -> Res::Resolved<()>
    where
        Res: QueryResultDistinct,
    {
        let res = f(&self.index);
        res.map(|key| {
            self.data.update(key, |existing| {
                let new = update_fn(existing);
                self.index.update(
                    SEAL,
                    &Update {
                        key,
                        new: &new,
                        existing,
                    },
                );
                new
            });
        })
    }

    pub fn take<Res>(&mut self, f: impl FnOnce(&Ix) -> Res) -> Res::Resolved<In>
    where
        Res: QueryResultDistinct,
    {
        let res = f(&self.index);
        res.map(|k| self.delete_by_key(k).unwrap())
    }

    /// Number of items in the collection.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn mk_key(&mut self) -> Key {
        let k = Key {
            id: self.next_key_id,
        };
        self.next_key_id += 1;
        k
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TrivialIndex;
    impl<In> Index<In> for TrivialIndex {
        fn insert(&mut self, _seal: Seal, _op: &Insert<In>) {}
        fn remove(&mut self, _seal: Seal, _op: &Remove<In>) {}
    }

    #[test]
    fn test_len() {
        let mut collection = Collection::new(TrivialIndex);
        assert_eq!(collection.len(), 0);

        collection.insert(1);
        assert_eq!(collection.len(), 1);

        collection.insert(2);
        assert_eq!(collection.len(), 2);

        let key = collection.insert(3);
        assert_eq!(collection.len(), 3);

        collection.delete_by_key(key);
        assert_eq!(collection.len(), 2);
    }

    // See composable-indexes/src/lib.rs for more tests
}
