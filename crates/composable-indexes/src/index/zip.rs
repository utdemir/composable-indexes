#![allow(clippy::too_many_arguments)]

use crate::{
    Index,
    core::{Insert, Remove, Seal},
};

// Implementations

macro_rules! tuple_impl {
    ($n:literal) => {
        seq_macro::seq!(N in 0..$n {
            impl<In, #( Ix~N, )*> Index<In> for ( #( Ix~N, )* )
            where
                #( Ix~N: Index<In>, )*
            {
                #[inline]
                fn insert(&mut self, seal: Seal, op: &Insert<In>) {
                    #(self.N.insert(seal, op);)*
                }

                #[inline]
                fn remove(&mut self, seal: Seal, op: &Remove<In>) {
                    #(self.N.remove(seal, op);)*
                }
            }
        });
    };
}

tuple_impl!(2);
tuple_impl!(3);
tuple_impl!(4);
tuple_impl!(5);
tuple_impl!(6);
tuple_impl!(7);
tuple_impl!(8);
tuple_impl!(9);
tuple_impl!(10);
tuple_impl!(11);
tuple_impl!(12);
tuple_impl!(13);
tuple_impl!(14);
tuple_impl!(15);
tuple_impl!(16);

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;

    use crate::testutils::prop_assert_reference;

    use crate::{Collection, index};

    #[test]
    fn test_zip() {
        let ix0 = index::BTree::<i32>::new();
        let ix1 = index::BTree::<i32>::new();
        let ix2 = index::BTree::<i32>::new();
        let ix3 = index::BTree::<i32>::new();
        let ix4 = index::BTree::<i32>::new();

        let ix = (ix0, ix1, ix2, ix3, ix4);

        let mut db = Collection::<i32, _>::new(ix);

        db.insert(1);
        db.insert(2);

        db.query(|ix| ix.4.get_one(&1));
    }

    #[test]
    fn test_reference() {
        prop_assert_reference(
            || (index::BTree::<u8>::new(), index::BTree::<u8>::new()),
            |db| {
                let (c, m) = db.query(|ix| (ix.0.count_distinct(), ix.1.max_one()));
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
