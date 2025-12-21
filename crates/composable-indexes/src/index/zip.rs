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
//!      index::premap(|p: &Person| p.name.clone(), index::btree()),
//!    )
//! );
//!
//! cs.query(|ix| ix._1().max_one());
//! cs.query(|ix| ix._2().get_one(&"Alice".to_string()));
//! ```

use paste::paste;
use seq_macro::seq;

macro_rules! generate_zip_variant {
    ($n:literal) => {
        seq_macro::seq!(N in 1..=$n {
            paste! {
                #[doc = "Zips " $n " indexes into a single index"]
                #[allow(clippy::too_many_arguments)]
                pub fn [<zip $n>]<In, #( Ix~N, )*>(
                    #( ix~N: Ix~N, )*
                ) -> [<ZipIndex $n>]<In, #( Ix~N, )*> {
                    [<ZipIndex $n>] {
                        #( ix~N, )*
                        _marker: core::marker::PhantomData,
                    }
                }

                pub struct [<ZipIndex $n>]<In, #( Ix~N, )*> {
                    #( ix~N: Ix~N, )*
                    _marker: core::marker::PhantomData<fn() -> In>,
                }

                impl<In, #( Ix~N, )*> Clone for [<ZipIndex $n>]<In, #( Ix~N, )*>
                where
                    #( Ix~N: Clone, )*
                {
                    fn clone(&self) -> Self {
                        [<ZipIndex $n>] {
                            #( ix~N: self.ix~N.clone(), )*
                            _marker: core::marker::PhantomData,
                        }
                    }
                }

                impl<In, #( Ix~N: crate::ShallowClone, )*> crate::ShallowClone for [<ZipIndex $n>]<In, #( Ix~N, )*> {}

                impl<In, #( Ix~N, )*> crate::core::Index<In> for [<ZipIndex $n>]<In, #( Ix~N, )*>
                where
                    #( Ix~N: crate::core::Index<In>, )*
                {
                    #[inline]
                    fn insert(&mut self, op: &crate::core::Insert<In>) {
                        #(self.ix~N.insert(op);)*
                    }

                    #[inline]
                    fn update(&mut self, op: &crate::core::Update<In>) {
                        #(self.ix~N.update(op);)*
                    }

                    #[inline]
                    fn remove(&mut self, op: &crate::core::Remove<In>) {
                        #(self.ix~N.remove(op);)*
                    }
                }

                impl<In, #( Ix~N, )*> [<ZipIndex $n>]<In, #( Ix~N, )*> {
                    #(
                        #[allow(non_snake_case)]
                        #[inline]
                        pub fn [< _~N >](&self) -> &Ix~N {
                            &self.ix~N
                        }
                    )*
                }
            }
        });
    };
}

seq!(N in 2..=10 {
    generate_zip_variant!(N);
});

pub use crate::zip;

#[macro_export]
#[doc(hidden)]
macro_rules! zip {
    ($($ix:expr),*) => {
        $ix
    };
    ($ix1:expr, $ix2:expr $(,)?) => {
        $crate::index::zip::zip2($ix1, $ix2)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr $(,)?) => {
        $crate::index::zip::zip3($ix1, $ix2, $ix3)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr $(,)?) => {
        $crate::index::zip::zip4($ix1, $ix2, $ix3, $ix4)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr, $ix5:expr $(,)?) => {
        $crate::index::zip::zip5($ix1, $ix2, $ix3, $ix4, $ix5)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr, $ix5:expr, $ix6:expr $(,)?) => {
        $crate::index::zip::zip6($ix1, $ix2, $ix3, $ix4, $ix5, $ix6)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr, $ix5:expr, $ix6:expr, $ix7:expr $(,)?) => {
        $crate::index::zip::zip7($ix1, $ix2, $ix3, $ix4, $ix5, $ix6, $ix7)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr, $ix5:expr, $ix6:expr, $ix7:expr, $ix8:expr $(,)?) => {
        $crate::index::zip::zip8($ix1, $ix2, $ix3, $ix4, $ix5, $ix6, $ix7, $ix8)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr, $ix5:expr, $ix6:expr, $ix7:expr, $ix8:expr, $ix9:expr $(,)?) => {
        $crate::index::zip::zip9($ix1, $ix2, $ix3, $ix4, $ix5, $ix6, $ix7, $ix8, $ix9)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr, $ix5:expr, $ix6:expr, $ix7:expr, $ix8:expr, $ix9:expr, $ix10:expr $(,)?) => {
        $crate::index::zip::zip10($ix1, $ix2, $ix3, $ix4, $ix5, $ix6, $ix7, $ix8, $ix9, $ix10)
    };
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;

    use crate::testutils::prop_assert_reference;

    use super::*;
    use crate::Collection;
    use crate::index::btree::btree;

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

        db.query(|ix| ix._5().get_one(&1));
    }

    #[test]
    fn test_reference() {
        prop_assert_reference(
            || zip2(btree::<u8>(), btree()),
            |db| {
                let (c, m) = db.query(|ix| (ix._1().count_distinct(), ix._2().max_one()));
                (c, m.cloned())
            },
            |xs| {
                let count = xs.iter().copied().collect::<BTreeSet<_>>().len();
                let max = xs.iter().max().cloned();
                (count, max)
            },
            None,
        )
    }
}
