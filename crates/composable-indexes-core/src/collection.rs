use std::collections::HashMap;

use crate::index::{Index, QueryEnv};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    pub id: u64,
}

/// A collection of items, with an index that is automatically kept up-to-date.
pub struct Collection<In, Ix> {
    index: Ix,
    data: HashMap<Key, In>,
    next_key_id: u64,
}

impl<In, Ix> Collection<In, Ix>
where
    Ix: Index<In>,
{
    /// Create an empty collection.
    pub fn new(ix: Ix) -> Self {
        Collection {
            data: HashMap::new(),
            next_key_id: 0,
            index: ix,
        }
    }

    /// Lookup an item in the collection by its key.
    pub fn get(&self, key: Key) -> Option<&In> {
        self.data.get(&key)
    }

    /// Insert a new item into the collection.
    pub fn insert(&mut self, value: In) -> Key {
        let key = self.mk_key();
        let existing = self.data.insert(key, value);

        // There shouldn't be an existing key, as we just generated it
        debug_assert!(existing.is_none());

        self.index.insert(&Insert {
            key,
            new: &self.data[&key],
        });

        key
    }

    /// Iterate over all items in the collection.
    pub fn iter(&self) -> impl Iterator<Item = (&Key, &In)> {
        self.data.iter()
    }

    /// Mutate (or alter the presence of) the item in the collection.
    pub fn update_mut<F>(&mut self, key: Key, f: F)
    where
        F: FnOnce(&mut Option<In>),
    {
        let mut existing = self.delete(key);
        f(&mut existing);

        if let Some(existing) = existing {
            self.data.insert(key, existing);
            self.index.insert(&Insert {
                key,
                new: &self.data[&key],
            });
        }
    }

    /// Update the item in the collection.
    pub fn update<F>(&mut self, key: Key, f: F)
    where
        F: FnOnce(Option<&In>) -> In,
    {
        let existing = self.data.get(&key);
        let new = f(existing);

        match existing {
            Some(existing) => {
                self.index.update(&Update {
                    key,
                    new: &new,
                    existing,
                });
                self.data.insert(key, new);
            }
            None => {
                self.index.insert(&Insert { key, new: &new });
                self.data.insert(key, new);
            }
        };
    }

    /// Mutate the item in the collection, if it exists.
    pub fn adjust_mut<F>(&mut self, key: Key, f: F)
    where
        F: FnOnce(&mut In),
    {
        if let Some(mut existing) = self.delete(key) {
            f(&mut existing);
            self.data.insert(key, existing);
            self.index.insert(&Insert {
                key,
                new: &self.data[&key],
            });
        }
    }

    /// Adjust the item in the collection, if it exists.
    pub fn adjust<F>(&mut self, key: Key, f: F)
    where
        F: FnOnce(&In) -> In,
    {
        if let Some(existing) = self.data.get(&key) {
            let new = f(existing);
            self.index.update(&Update {
                key,
                new: &new,
                existing,
            });
            self.data.insert(key, new);
        }
    }

    /// Remove an item from the collection, returning it if it exists.
    pub fn delete(&mut self, key: Key) -> Option<In> {
        let existing = self.data.remove(&key);
        if let Some(ref existing) = existing {
            self.index.remove(&Remove { key, existing });
        }
        existing
    }

    /// Query the collection using its index(es).
    pub fn query<'t>(&'t self) -> Ix::Query<'t, In> {
        let env = QueryEnv { data: &self.data };
        self.index.query(env)
    }

    /// Number of items in the collection.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    fn mk_key(&mut self) -> Key {
        let k = Key {
            id: self.next_key_id,
        };
        self.next_key_id += 1;
        k
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
