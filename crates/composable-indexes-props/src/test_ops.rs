use std::collections::{HashMap, HashSet};

use proptest::prelude::Arbitrary;

use composable_indexes::{Collection, Index, Key};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DBOperation<T> {
    InsertOrUpdate(Key, T),
    Delete(Key),
}

#[derive(Debug, Clone)]
pub struct TestOps<T: Clone> {
    pub operations: Vec<DBOperation<T>>,
}

impl<T: Clone> TestOps<T> {
    pub fn apply<'t, Ix: Index<'t, T>>(&self, db: &mut Collection<T, Ix>) {
        self.operations.iter().cloned().for_each(|op| match op {
            DBOperation::InsertOrUpdate(key, value) => {
                db.update(key, |_existing| value);
            }
            DBOperation::Delete(key) => {
                db.delete(key);
            }
        });
    }

    pub fn end_state(&self) -> HashMap<Key, T> {
        let mut ret = HashMap::new();

        self.operations.iter().for_each(|op| match op {
            DBOperation::InsertOrUpdate(key, value) => {
                ret.insert(key.clone(), value.clone());
            }
            DBOperation::Delete(key) => {
                ret.remove(key);
            }
        });

        ret
    }
}

impl<T: Arbitrary + Clone + 'static> proptest::arbitrary::Arbitrary for TestOps<T> {
    type Strategy = proptest::strategy::BoxedStrategy<Self>;
    type Parameters = ();

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        use proptest::prelude::*;
        prop::collection::vec((0u32..50, any::<Option<T>>()), 0..400)
            .prop_map(|ops| {
                // Ensure the first operation per key is Some
                let mut seen_keys: HashSet<u32> = HashSet::new();
                let mut operations = Vec::new();
                for (key, value) in ops {
                    if seen_keys.insert(key) {
                        if value.is_some() {
                            operations.push((key, value));
                        } else {
                            // Ignore if the first value is some
                        }
                    } else {
                        operations.push((key, None));
                    }
                }

                operations
            })
            .prop_map(|ops| {
                let ops = ops
                    .into_iter()
                    .map(|(k, v)| {
                        let k = Key { id: k.into() };
                        if let Some(v) = v {
                            DBOperation::<T>::InsertOrUpdate(k, v)
                        } else {
                            DBOperation::Delete(k)
                        }
                    })
                    .collect::<Vec<DBOperation<T>>>();

                TestOps { operations: ops }
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use composable_indexes::indexes;

    #[proptest::property_test]
    fn test_test_ops(ops: TestOps<String>) {
        let mut db = Collection::<String, _>::new(indexes::btree());
        ops.apply(&mut db);

        let expected = ops.end_state();
        let actual = db
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<HashMap<Key, String>>();

        assert_eq!(expected, actual);
    }
}
