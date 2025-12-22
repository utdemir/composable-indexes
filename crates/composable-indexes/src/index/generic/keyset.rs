use crate::Key;
#[cfg(feature = "imbl")]
use crate::ShallowClone;

pub type DefaultKeySet = hashbrown::HashSet<Key>;

#[cfg(feature = "imbl")]
pub type DefaultImmutableKeySet = imbl::OrdSet<Key>;

pub trait KeySet: Default {
    type Iter<'a>: Iterator<Item = Key>
    where
        Self: 'a;

    fn insert(&mut self, key: Key);
    fn remove(&mut self, key: &Key);
    fn contains(&self, key: &Key) -> bool;
    fn iter(&self) -> Self::Iter<'_>;
    fn is_empty(&self) -> bool;
    fn count(&self) -> usize;
}

impl KeySet for alloc::collections::BTreeSet<Key> {
    type Iter<'a>
        = std::iter::Copied<alloc::collections::btree_set::Iter<'a, Key>>
    where
        Self: 'a;

    fn insert(&mut self, key: Key) {
        alloc::collections::BTreeSet::insert(self, key);
    }

    fn remove(&mut self, key: &Key) {
        alloc::collections::BTreeSet::remove(self, key);
    }

    fn contains(&self, key: &Key) -> bool {
        alloc::collections::BTreeSet::contains(self, key)
    }

    fn iter(&self) -> Self::Iter<'_> {
        alloc::collections::BTreeSet::iter(self).copied()
    }

    fn is_empty(&self) -> bool {
        alloc::collections::BTreeSet::is_empty(self)
    }

    fn count(&self) -> usize {
        alloc::collections::BTreeSet::len(self)
    }
}

impl<S: core::hash::BuildHasher + Default> KeySet for hashbrown::HashSet<Key, S> {
    type Iter<'a>
        = std::iter::Copied<hashbrown::hash_set::Iter<'a, Key>>
    where
        Self: 'a;

    fn insert(&mut self, key: Key) {
        hashbrown::HashSet::insert(self, key);
    }

    fn remove(&mut self, key: &Key) {
        hashbrown::HashSet::remove(self, key);
    }

    fn contains(&self, key: &Key) -> bool {
        hashbrown::HashSet::contains(self, key)
    }

    fn iter(&self) -> Self::Iter<'_> {
        hashbrown::HashSet::iter(self).copied()
    }

    fn is_empty(&self) -> bool {
        hashbrown::HashSet::is_empty(self)
    }

    fn count(&self) -> usize {
        hashbrown::HashSet::len(self)
    }
}

#[cfg(feature = "std")]
impl<S: core::hash::BuildHasher + Default> KeySet for std::collections::HashSet<Key, S> {
    type Iter<'a>
        = std::iter::Copied<std::collections::hash_set::Iter<'a, Key>>
    where
        Self: 'a;

    fn insert(&mut self, key: Key) {
        std::collections::HashSet::insert(self, key);
    }

    fn remove(&mut self, key: &Key) {
        std::collections::HashSet::remove(self, key);
    }

    fn contains(&self, key: &Key) -> bool {
        std::collections::HashSet::contains(self, key)
    }

    fn iter(&self) -> Self::Iter<'_> {
        std::collections::HashSet::iter(self).copied()
    }

    fn is_empty(&self) -> bool {
        std::collections::HashSet::is_empty(self)
    }

    fn count(&self) -> usize {
        std::collections::HashSet::len(self)
    }
}

#[cfg(feature = "imbl")]
impl KeySet for imbl::OrdSet<Key> {
    type Iter<'a>
        = std::iter::Copied<imbl::ordset::Iter<'a, Key, imbl::shared_ptr::DefaultSharedPtr>>
    where
        Self: 'a;

    fn insert(&mut self, key: Key) {
        imbl::OrdSet::insert(self, key);
    }

    fn remove(&mut self, key: &Key) {
        imbl::OrdSet::remove(self, key);
    }

    fn contains(&self, key: &Key) -> bool {
        imbl::OrdSet::contains(self, key)
    }

    fn iter(&self) -> Self::Iter<'_> {
        imbl::OrdSet::iter(self).copied()
    }

    fn is_empty(&self) -> bool {
        imbl::OrdSet::is_empty(self)
    }

    fn count(&self) -> usize {
        imbl::OrdSet::len(self)
    }
}

#[cfg(feature = "imbl")]
impl ShallowClone for imbl::OrdSet<Key> {}

#[cfg(feature = "imbl")]
impl KeySet for imbl::HashSet<Key> {
    type Iter<'a>
        = std::iter::Copied<imbl::hashset::Iter<'a, Key, imbl::shared_ptr::DefaultSharedPtr>>
    where
        Self: 'a;

    fn insert(&mut self, key: Key) {
        imbl::HashSet::insert(self, key);
    }

    fn remove(&mut self, key: &Key) {
        imbl::HashSet::remove(self, key);
    }

    fn contains(&self, key: &Key) -> bool {
        imbl::HashSet::contains(self, key)
    }

    fn iter(&self) -> Self::Iter<'_> {
        imbl::HashSet::iter(self).copied()
    }

    fn is_empty(&self) -> bool {
        imbl::HashSet::is_empty(self)
    }

    fn count(&self) -> usize {
        imbl::HashSet::len(self)
    }
}

#[cfg(feature = "imbl")]
impl ShallowClone for imbl::HashSet<Key> {}

#[cfg(feature = "roaring")]
impl KeySet for roaring::RoaringTreemap {
    type Iter<'a>
        = RoaringIter<'a>
    where
        Self: 'a;

    fn insert(&mut self, key: Key) {
        roaring::RoaringTreemap::insert(self, key.id);
    }

    fn remove(&mut self, key: &Key) {
        roaring::RoaringTreemap::remove(self, key.id);
    }

    fn contains(&self, key: &Key) -> bool {
        roaring::RoaringTreemap::contains(self, key.id)
    }

    fn iter(&self) -> Self::Iter<'_> {
        RoaringIter {
            inner: roaring::RoaringTreemap::iter(self),
        }
    }

    fn is_empty(&self) -> bool {
        roaring::RoaringTreemap::is_empty(self)
    }

    fn count(&self) -> usize {
        roaring::RoaringTreemap::len(self) as usize
    }
}

#[cfg(feature = "roaring")]
pub struct RoaringIter<'a> {
    inner: roaring::treemap::Iter<'a>,
}

#[cfg(feature = "roaring")]
impl<'a> Iterator for RoaringIter<'a> {
    type Item = Key;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|id| Key { id })
    }
}
