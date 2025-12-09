//! An index that maintains the keys of all received items.

use std::collections::HashSet;

use composable_indexes_core::{Index, Insert, Key, Remove};

pub fn keys() -> KeysIndex {
    KeysIndex {
        keys: HashSet::new(),
    }
}

pub struct KeysIndex {
    pub keys: HashSet<Key>,
}

impl<In> Index<In> for KeysIndex {
    fn insert(&mut self, op: &Insert<In>) {
        self.keys.insert(op.key);
    }
    fn remove(&mut self, op: &Remove<In>) {
        self.keys.remove(&op.key);
    }
}
impl KeysIndex {
    pub fn all(&self) -> impl Iterator<Item = Key> {
        self.keys.iter().copied()
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.keys.contains(key)
    }

    pub fn count(&self) -> usize {
        self.keys.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use composable_indexes_core::Collection;

    #[test]
    fn test_basic() {
        let mut coll = Collection::<u8, _>::new(keys());

        let key1 = coll.insert(1);
        let key2 = coll.insert(2);

        assert!(coll.query(|ix| ix.contains(&key1)));
        assert!(coll.query(|ix| ix.contains(&key2)));
        assert_eq!(coll.query(|ix| ix.count()), 2);
    }
}
