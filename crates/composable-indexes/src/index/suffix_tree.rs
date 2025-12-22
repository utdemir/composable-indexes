use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use hashbrown::HashSet;

use crate::{
    Key,
    core::Index,
    index::generic::{DefaultKeySet, KeySet},
};

pub fn suffix_tree() -> SuffixTreeIndex<DefaultKeySet> {
    SuffixTreeIndex {
        suffix_tree: BTreeMap::new(),
    }
}

pub struct SuffixTreeIndex<KeySet_ = DefaultKeySet> {
    suffix_tree: BTreeMap<Suffix<'static>, KeySet_>,
}

impl<KeySet_> Index<String> for SuffixTreeIndex<KeySet_>
where
    KeySet_: crate::index::generic::KeySet,
{
    #[inline]
    fn insert(&mut self, op: &crate::core::Insert<String>) {
        let suffixes = Suffix::all_suffixes(op.new);
        for suffix in suffixes {
            self.suffix_tree.entry(suffix).or_default().insert(op.key);
        }
    }

    #[inline]
    fn remove(&mut self, op: &crate::core::Remove<String>) {
        let suffixes = Suffix::all_suffixes(op.existing);
        for suffix in suffixes {
            let key_set = self.suffix_tree.get_mut(&suffix).unwrap();
            key_set.remove(&op.key);
            if key_set.is_empty() {
                self.suffix_tree.remove(&suffix);
            }
        }
    }
}

impl<KeySet_> SuffixTreeIndex<KeySet_>
where
    KeySet_: KeySet,
{
    pub fn contains_get_all(&self, pattern: &str) -> HashSet<Key> {
        let suffix = Suffix::Ref { suffix: pattern };
        self.suffix_tree
            .range(suffix..)
            .next()
            .and_then(|(suffix, key_set)| {
                if suffix.as_ref().starts_with(pattern) {
                    Some(key_set.iter().collect())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    pub fn contains_get_one(&self, pattern: &str) -> Option<Key> {
        let suffix = Suffix::Ref { suffix: pattern };
        self.suffix_tree
            .range(suffix..)
            .next()
            .and_then(|(suffix, key_set)| {
                if suffix.as_ref().starts_with(pattern) {
                    key_set.iter().next()
                } else {
                    None
                }
            })
    }

    pub fn ends_with_get_one(&self, pattern: &str) -> Option<Key> {
        let suffix = Suffix::Ref { suffix: pattern };
        self.suffix_tree
            .get(&suffix)
            .and_then(|key_set| key_set.iter().next())
    }

    pub fn ends_with_get_all(&self, pattern: &str) -> HashSet<Key> {
        let suffix = Suffix::Ref { suffix: pattern };
        self.suffix_tree
            .get(&suffix)
            .map(|key_set| key_set.iter().collect())
            .unwrap_or_default()
    }

    pub fn count_distinct_suffixes(&self) -> usize {
        self.suffix_tree.len()
    }
}

enum Suffix<'a> {
    Owned { base: Rc<String>, index: usize },
    Ref { suffix: &'a str },
}

impl AsRef<str> for Suffix<'_> {
    fn as_ref(&self) -> &str {
        match self {
            Suffix::Owned { base, index } => &base[*index..],
            Suffix::Ref { suffix } => suffix,
        }
    }
}

impl Suffix<'_> {
    fn all_suffixes(s: &str) -> Vec<Suffix<'static>> {
        let mut suffixes = Vec::new();

        let base = Rc::new(s.to_string());
        for (i, _) in s.char_indices() {
            suffixes.push(Suffix::Owned {
                base: Rc::clone(&base),
                index: i,
            });
        }
        suffixes
    }
}

impl PartialEq for Suffix<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for Suffix<'_> {}

impl PartialOrd for Suffix<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl Ord for Suffix<'_> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use crate::testutils::{SortedVec, prop_assert_reference};

    use super::*;

    #[test]
    fn test_contains_ref() {
        prop_assert_reference(
            suffix_tree,
            |db| {
                db.query(|ix| ix.contains_get_all("aaa"))
                    .into_iter()
                    .cloned()
                    .collect::<SortedVec<_>>()
            },
            |data| {
                data.iter()
                    .filter(|s| s.contains("aaa"))
                    .cloned()
                    .collect::<SortedVec<_>>()
            },
            None,
        );
    }
}
