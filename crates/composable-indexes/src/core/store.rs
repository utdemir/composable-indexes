use crate::Key;

pub trait Store<T: 'static> {
    fn get(&self, key: Key) -> Option<&T>;

    fn get_unwrapped(&self, key: Key) -> &T {
        self.get(key)
            .expect("invariant violation: Key not found in store")
    }

    fn insert(&mut self, key: Key, value: T) -> Option<T>;
    fn remove(&mut self, key: Key) -> Option<T>;
    fn update<F>(&mut self, key: Key, f: F)
    where
        F: FnOnce(&T) -> T,
    {
        if let Some(existing) = self.get(key) {
            let new = f(existing);
            self.insert(key, new);
        }
    }

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> impl IntoIterator<Item = (Key, &T)>;
}

impl<T: 'static> Store<T> for alloc::collections::BTreeMap<Key, T> {
    fn get(&self, key: Key) -> Option<&T> {
        self.get(&key)
    }

    fn insert(&mut self, key: Key, value: T) -> Option<T> {
        self.insert(key, value)
    }

    fn remove(&mut self, key: Key) -> Option<T> {
        self.remove(&key)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn iter(&self) -> impl IntoIterator<Item = (Key, &T)> {
        self.iter().map(|(k, v)| (*k, v))
    }
}

#[cfg(feature = "std")]
impl<T: 'static> Store<T> for std::collections::HashMap<Key, T> {
    fn get(&self, key: Key) -> Option<&T> {
        self.get(&key)
    }

    fn insert(&mut self, key: Key, value: T) -> Option<T> {
        self.insert(key, value)
    }

    fn remove(&mut self, key: Key) -> Option<T> {
        self.remove(&key)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn iter(&self) -> impl IntoIterator<Item = (Key, &T)> {
        self.iter().map(|(k, v)| (*k, v))
    }
}
