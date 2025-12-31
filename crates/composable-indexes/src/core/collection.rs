use crate::{
    ShallowClone,
    core::{DefaultStore, SEAL, store::Store},
};

use super::{
    QueryResult, QueryResultDistinct,
    index::{Index, Insert, Remove, Update},
};

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
        Collection {
            data: S::default(),
            next_key_id: 0,
            index: ix,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<In, Ix, S> Collection<In, Ix, S>
where
    S: Store<In>,
    In: 'static,
    Ix: Index<In>,
{
    /// Create a collection with a custom store.
    ///
    /// If the store is non-empty, the indexes will be populated from the store's contents. So
    /// this may be an expensive operation depending on the size of the store.
    pub fn new_with_store(store: S, mut ix: Ix) -> Self {
        // Get the indexes up-to-date with the initial store contents
        let mut max_key_id = 0;
        for (k, v) in store.iter() {
            max_key_id = max_key_id.max(k.id);
            ix.insert(SEAL, &Insert { key: k, new: v });
        }

        // Then create the collection
        Collection {
            data: store,
            next_key_id: max_key_id + 1,
            index: ix,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<In, Ix, S> Collection<In, Ix, S> {
    /// Access the underlying store.
    pub fn store(&self) -> &S {
        &self.data
    }

    /// Extract the underlying store.
    pub fn extract_store(self) -> S {
        self.data
    }

    /// Mutably access the underlying store.
    /// Warning: Mutating the elements inside the store may lead to inconsistencies with the index
    /// and cause undefined behavior. Use with caution. Ideally - only use it in a way that doesn't
    /// modify the elements.
    pub fn unsafe_mut_store(&mut self) -> &mut S {
        &mut self.data
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
        I: Iterator<Item = In>,
    {
        for item in iter {
            self.insert(item);
        }
    }

    /// Iterate over all items in the collection.
    pub fn iter(&self) -> impl Iterator<Item = (Key, &In)> {
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

    /// Perform a query using the collection's indexes, returning references to the items.
    pub fn query<Res>(&self, f: impl FnOnce(&Ix) -> Res) -> Res::Resolved<&In>
    where
        Res: QueryResult,
    {
        let res = f(&self.index);
        res.map(|k| self.data.get_unwrapped(k))
    }

    /// Perform a query using the collection's indexes, returning the keys of the items.
    pub fn query_keys<Res>(&self, f: impl FnOnce(&Ix) -> Res) -> Res::Resolved<Key>
    where
        Res: QueryResult,
    {
        let res = f(&self.index);
        res.map(|k| k)
    }

    /// Perform a query using the collection's indexes, returning both the keys and the items.
    pub fn query_with_keys<Res>(&self, f: impl FnOnce(&Ix) -> Res) -> Res::Resolved<(Key, &In)>
    where
        Res: QueryResult,
    {
        let res = f(&self.index);
        res.map(|k| (k, self.data.get_unwrapped(k)))
    }

    /// Delete all items matching the query.
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

    /// Update all items matching the query with the provided function.
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

    /// Remove and return all items matching the query.
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

    pub fn vacuum(&mut self) {
        self.data.vacuum();
        self.index.vacuum(SEAL);
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

    use crate::core::{Index, Insert, Plain, Remove, Seal, Update};
    use alloc::vec::Vec;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Op<T> {
        Insert(u64, T),
        Update(u64, T, T),
        Remove(u64, T),
    }

    pub struct TestIndex<T: Clone> {
        pub ops: Vec<Op<T>>,
    }

    impl<T: Clone> TestIndex<T> {
        pub fn new() -> Self {
            Self { ops: Vec::new() }
        }
    }
    impl<T: Clone> Index<T> for TestIndex<T> {
        fn insert(&mut self, _seal: Seal, op: &Insert<T>) {
            self.ops.push(Op::Insert(op.key.id, op.new.clone()));
        }

        fn update(&mut self, _seal: Seal, op: &Update<T>) {
            self.ops
                .push(Op::Update(op.key.id, op.existing.clone(), op.new.clone()));
        }

        fn remove(&mut self, _seal: Seal, op: &Remove<T>) {
            self.ops.push(Op::Remove(op.key.id, op.existing.clone()));
        }
    }

    #[test]
    fn test_len() {
        let mut collection = Collection::new(());
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

    #[test]
    fn simple() {
        let mut db = Collection::<u32, _>::new(TestIndex::new());

        let one = db.insert(1);
        let two = db.insert(2);
        let three = db.insert(3);
        db.update_by_key(two, |_| 10);
        let four = db.insert(4);
        db.delete_by_key(three);

        assert_eq!(db.get_by_key(one), Some(&1));
        assert_eq!(db.get_by_key(two), Some(&10));
        assert_eq!(db.get_by_key(three), None);
        assert_eq!(db.get_by_key(four), Some(&4));
        assert_eq!(db.len(), 3);

        // Access test index operations directly
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(
            ops,
            vec![
                Op::Insert(0, 1),
                Op::Insert(1, 2),
                Op::Insert(2, 3),
                Op::Update(1, 2, 10),
                Op::Insert(3, 4),
                Op::Remove(2, 3),
            ]
        );
    }

    #[test]
    fn update_mut_updates() {
        let mut db = Collection::<u32, _>::new(TestIndex::new());

        let one = db.insert(1);
        db.update_by_key_mut(one, |v| {
            if let Some(v) = v {
                *v += 1;
            }
        });

        assert_eq!(db.get_by_key(one), Some(&2));
        assert_eq!(db.len(), 1);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(
            ops,
            vec![Op::Insert(0, 1), Op::Remove(0, 1), Op::Insert(0, 2),]
        );
    }

    #[test]
    fn update_mut_inserts() {
        let mut db = Collection::<u32, _>::new(TestIndex::new());

        let one = db.insert(1);
        db.delete_by_key(one);
        db.update_by_key_mut(one, |v| {
            assert!(v.is_none());
            *v = Some(2);
        });

        assert_eq!(db.get_by_key(one), Some(&2));
        assert_eq!(db.len(), 1);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(
            ops,
            vec![Op::Insert(0, 1), Op::Remove(0, 1), Op::Insert(0, 2),]
        );
    }

    #[test]
    fn update_mut_removes() {
        let mut db = Collection::<u32, _>::new(TestIndex::new());

        let one = db.insert(1);
        db.update_by_key_mut(one, |v| {
            assert!(v.is_some());
            *v = None;
        });

        assert_eq!(db.get_by_key(one), None);
        assert_eq!(db.len(), 0);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(ops, vec![Op::Insert(0, 1), Op::Remove(0, 1),]);
    }

    #[test]
    fn update_updates() {
        let mut db = Collection::<u32, _>::new(TestIndex::new());

        let one = db.insert(1);
        db.update_by_key(one, |_| 2);

        assert_eq!(db.get_by_key(one), Some(&2));
        assert_eq!(db.len(), 1);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(ops, vec![Op::Insert(0, 1), Op::Update(0, 1, 2),]);
    }

    #[test]
    fn update_inserts() {
        let mut db = Collection::<u32, _>::new(TestIndex::new());

        let one = db.insert(1);
        db.delete_by_key(one);

        db.update_by_key(one, |x| {
            assert_eq!(x, None);
            2
        });

        assert_eq!(db.get_by_key(one), Some(&2));
        assert_eq!(db.len(), 1);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(
            ops,
            vec![Op::Insert(0, 1), Op::Remove(0, 1), Op::Insert(0, 2),]
        );
    }

    #[test]
    fn adjust_mut_updates() {
        let mut db = Collection::<u32, _>::new(TestIndex::new());

        let one = db.insert(1);
        db.adjust_by_key_mut(one, |v| {
            *v = 2;
        });

        assert_eq!(db.get_by_key(one), Some(&2));
        assert_eq!(db.len(), 1);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(
            ops,
            vec![Op::Insert(0, 1), Op::Remove(0, 1), Op::Insert(0, 2),]
        );
    }

    #[test]
    fn adjust_mut_ignores_non_existent() {
        let mut db = Collection::<u32, _>::new(TestIndex::new());

        let one = db.insert(1);
        db.delete_by_key(one);

        db.adjust_by_key_mut(one, |_| {
            panic!("Should not be called");
        });

        assert_eq!(db.get_by_key(one), None);
        assert_eq!(db.len(), 0);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(ops, vec![Op::Insert(0, 1), Op::Remove(0, 1),]);
    }

    #[test]
    fn adjust_updates() {
        let mut db = Collection::<u32, _>::new(TestIndex::new());

        let one = db.insert(1);
        db.adjust_by_key(one, |_| 2);

        assert_eq!(db.get_by_key(one), Some(&2));
        assert_eq!(db.len(), 1);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(ops, vec![Op::Insert(0, 1), Op::Update(0, 1, 2),]);
    }

    #[test]
    fn adjust_ignores_non_existent() {
        let mut db = Collection::<u32, _>::new(TestIndex::new());

        let one = db.insert(1);
        db.delete_by_key(one);

        db.adjust_by_key(one, |_| 2);

        assert_eq!(db.get_by_key(one), None);
        assert_eq!(db.len(), 0);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(ops, vec![Op::Insert(0, 1), Op::Remove(0, 1),]);
    }
}
