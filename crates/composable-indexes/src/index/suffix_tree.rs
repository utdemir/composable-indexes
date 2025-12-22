use alloc::collections::BTreeMap;
use alloc::rc::Rc;

use crate::{Key, core::Index, index::generic::{DefaultKeySet, KeySet}};

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
    where KeySet_: KeySet {
    pub fn search_all(&self, pattern: &str) -> Vec<Key> {
        let suffix = Suffix::Ref { suffix: pattern };
        self.suffix_tree.range(suffix..).next().and_then(|(suffix, key_set)| {
            if suffix.as_ref().starts_with(pattern) {
                Some(key_set.iter().copied().collect())
            } else {
                None
            }
        }).unwrap_or_default()
    }

    pub fn search_one(&self, pattern: &str) -> Option<Key> {
        let suffix = Suffix::Ref { suffix: pattern };
        self.suffix_tree.range(suffix..).next().and_then(|(suffix, key_set)| {
            if suffix.as_ref().starts_with(pattern) {
                key_set.iter().copied().next()
            } else {
                None
            }
        })
    }
}

// Suffix

enum Suffix<'a> {
    Owned {
        base: Rc<String>,
        index: usize,
    },
    Ref {
        suffix: &'a str,
    },
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
        for i in 0..s.len() {
            suffixes.push(
                Suffix::Owned {
                    base: Rc::clone(&base),
                    index: i,
                }
            );
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
