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
//!      index::PremapOwned::new(|p: &Person| p.age, index::BTree::<u32>::new()),
//!      index::PremapOwned::new(|p: &Person| p.name.clone(), index::BTree::<String>::new()),
//!    )
//! );
//!
//! cs.query(|ix| ix._1().max_one());
//! cs.query(|ix| ix._2().get_one(&"Alice".to_string()));
//! ```

#![allow(clippy::too_many_arguments)]

use paste::paste;
use seq_macro::seq;

macro_rules! generate_zip_variant {
    ($n:literal) => {
        seq_macro::seq!(N in 1..=$n {
            paste! {
                #[doc = "Zips " $n " indexes into a single index"]
                pub struct [<Zip $n>]<In, #( Ix~N, )*> {
                    #( ix~N: Ix~N, )*
                    _marker: core::marker::PhantomData<fn() -> In>,
                }

                impl<In, #( Ix~N, )*> Clone for [<Zip $n>]<In, #( Ix~N, )*>
                where
                    #( Ix~N: Clone, )*
                {
                    fn clone(&self) -> Self {
                        [<Zip $n>] {
                            #( ix~N: self.ix~N.clone(), )*
                            _marker: core::marker::PhantomData,
                        }
                    }
                }

                impl<In, #( Ix~N: crate::ShallowClone, )*> crate::ShallowClone for [<Zip $n>]<In, #( Ix~N, )*> {}

                impl<In, #( Ix~N, )*> crate::core::Index<In> for [<Zip $n>]<In, #( Ix~N, )*>
                where
                    #( Ix~N: crate::core::Index<In>, )*
                {
                    #[inline]
                    fn insert(&mut self, seal: crate::core::Seal, op: &crate::core::Insert<In>) {
                        #(self.ix~N.insert(seal, op);)*
                    }

                    #[inline]
                    fn update(&mut self, seal: crate::core::Seal, op: &crate::core::Update<In>) {
                        #(self.ix~N.update(seal, op);)*
                    }

                    #[inline]
                    fn remove(&mut self, seal: crate::core::Seal, op: &crate::core::Remove<In>) {
                        #(self.ix~N.remove(seal, op);)*
                    }
                }

                impl<In, #( Ix~N, )*> [<Zip $n>]<In, #( Ix~N, )*> {
                    pub fn new(#( ix~N: Ix~N, )*) -> Self {
                        [<Zip $n>] {
                            #( ix~N, )*
                            _marker: core::marker::PhantomData,
                        }
                    }

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
        $crate::index::Zip2::new($ix1, $ix2)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr $(,)?) => {
        $crate::index::Zip3::new($ix1, $ix2, $ix3)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr $(,)?) => {
        $crate::index::Zip4::new($ix1, $ix2, $ix3, $ix4)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr, $ix5:expr $(,)?) => {
        $crate::index::Zip5::new($ix1, $ix2, $ix3, $ix4, $ix5)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr, $ix5:expr, $ix6:expr $(,)?) => {
        $crate::index::Zip6::new($ix1, $ix2, $ix3, $ix4, $ix5, $ix6)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr, $ix5:expr, $ix6:expr, $ix7:expr $(,)?) => {
        $crate::index::Zip7::new($ix1, $ix2, $ix3, $ix4, $ix5, $ix6, $ix7)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr, $ix5:expr, $ix6:expr, $ix7:expr, $ix8:expr $(,)?) => {
        $crate::index::Zip8::new($ix1, $ix2, $ix3, $ix4, $ix5, $ix6, $ix7, $ix8)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr, $ix5:expr, $ix6:expr, $ix7:expr, $ix8:expr, $ix9:expr $(,)?) => {
        $crate::index::Zip9::new($ix1, $ix2, $ix3, $ix4, $ix5, $ix6, $ix7, $ix8, $ix9)
    };
    ($ix1:expr, $ix2:expr, $ix3:expr, $ix4:expr, $ix5:expr, $ix6:expr, $ix7:expr, $ix8:expr, $ix9:expr, $ix10:expr $(,)?) => {
        $crate::index::Zip10::new($ix1, $ix2, $ix3, $ix4, $ix5, $ix6, $ix7, $ix8, $ix9, $ix10)
    };
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;

    use crate::testutils::prop_assert_reference;

    use crate::{Collection, index};

    #[test]
    fn test_zip() {
        let ix1 = index::BTree::<i32>::new();
        let ix2 = index::BTree::<i32>::new();
        let ix3 = index::BTree::<i32>::new();
        let ix4 = index::BTree::<i32>::new();
        let ix5 = index::BTree::<i32>::new();

        let ix = super::Zip5::new(ix1, ix2, ix3, ix4, ix5);

        let mut db = Collection::<i32, _>::new(ix);

        db.insert(1);
        db.insert(2);

        db.query(|ix| ix._5().get_one(&1));
    }

    #[test]
    fn test_reference() {
        prop_assert_reference(
            || super::Zip2::new(index::BTree::<u8>::new(), index::BTree::<u8>::new()),
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
