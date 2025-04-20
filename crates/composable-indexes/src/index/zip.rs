//! Zips multiple indexes into a single index.
//!
//! For ease of use, you probably want to use the [`zip!`] macro
//! instead.
//!
//! ```
//! use composable_indexes::{Collection, index};
//!
//! struct Person { name: String, age: u32 }
//!
//! let cs = Collection::<Person, _>::new(
//!    index::zip!(
//!      index::premap(|p: &Person| p.age, index::btree()),
//!      index::premap(|p: &Person| p.name.clone(), index::hashtable()),
//!    )
//! );
//!
//! cs.query().0.max_one();
//! cs.query().1.get_one(&"Alice".to_string());
//! ```

use paste::paste;
use seq_macro::seq;

pub use composable_indexes_derive::zip;

macro_rules! generate_zip_variant {
    ($n:literal) => {
        seq_macro::seq!(N in 1..=$n {
            paste! {
                #[doc = "Zips " $n " indexes into a single index"]
                pub fn [<zip $n>]<In, #( Ix~N, )*>(
                    #( ix~N: Ix~N, )*
                ) -> [<ZipIndex $n>]<In, #( Ix~N, )*>
                where
                    #( Ix~N: composable_indexes_core::Index<In>, )*
                {
                    [<ZipIndex $n>] {
                        #( ix~N, )*
                        _marker: std::marker::PhantomData,
                    }
                }

                pub struct [<ZipIndex $n>]<In, #( Ix~N, )*> {
                    #( ix~N: Ix~N, )*
                    _marker: std::marker::PhantomData<In>,
                }

                impl<In, #( Ix~N, )*> composable_indexes_core::Index<In> for [<ZipIndex $n>]<In, #( Ix~N, )*>
                where
                    #( Ix~N: composable_indexes_core::Index<In>, )*
                {
                    type Query<'t, Out> = (#(Ix~N::Query<'t, Out>,)*)
                    where
                        Self: 't,
                        Out: 't;

                    fn insert(&mut self, op: &composable_indexes_core::Insert<In>) {
                        #(self.ix~N.insert(op);)*
                    }

                    fn update(&mut self, op: &composable_indexes_core::Update<In>) {
                        #(self.ix~N.update(op);)*
                    }

                    fn remove(&mut self, op: &composable_indexes_core::Remove<In>) {
                        #(self.ix~N.remove(op);)*
                    }

                    fn query<'t, Out: 't>(&'t self, env: composable_indexes_core::QueryEnv<'t, Out>) -> Self::Query<'t, Out> {
                        (#(self.ix~N.query(env.clone()),)*)
                    }
                }
            }
        });
    };
}

seq!(N in 2..=16 {
    generate_zip_variant!(N);
});

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use composable_indexes_testutils::prop_assert_reference;

    use super::*;
    use crate::Collection;
    use crate::index::btree::btree;
    use crate::index::hashtable;

    #[test]
    fn test_zip() {
        let ix1 = btree();
        let ix2 = btree();
        let ix3 = btree();
        let ix4 = btree();
        let ix5 = btree();

        let ix = zip5(ix1, ix2, ix3, ix4, ix5);

        let mut db = Collection::<i32, _>::new(ix);

        db.insert(1);
        db.insert(2);

        db.query().4.get_one(&1);
    }

    #[test]
    fn test_reference() {
        prop_assert_reference(
            || zip2(hashtable::<u8>(), btree()),
            |q| (q.0.count_distinct().clone(), q.1.max_one().cloned()),
            |xs| {
                (
                    xs.iter().map(|i| i.clone()).collect::<HashSet<_>>().len(),
                    xs.iter().max().cloned(),
                )
            },
            None,
        )
    }
}
