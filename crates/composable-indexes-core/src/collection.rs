use std::collections::HashMap;

use crate::index::{Index, QueryEnv};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    pub id: u64,
}

pub struct Collection<In, Ix> {
    index: Ix,
    data: HashMap<Key, In>,
    next_key_id: u64,
}

impl<'t, In, Ix> Collection<In, Ix>
where
    Ix: Index<'t, In>,
{
    pub fn new(ix: Ix) -> Self {
        Collection {
            data: HashMap::new(),
            next_key_id: 0,
            index: ix,
        }
    }

    pub fn get(&self, key: Key) -> Option<&In> {
        self.data.get(&key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Key, &In)> {
        self.data.iter()
    }

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

    pub fn delete(&mut self, key: Key) {
        match self.data.remove(&key) {
            Some(ref existing) => {
                self.index.remove(&Remove { key, existing });
            }
            None => {}
        }
    }

    pub fn query(&'t self) -> Ix::Query<In> {
        let env = QueryEnv { data: &self.data };
        self.index.query(env)
    }

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
