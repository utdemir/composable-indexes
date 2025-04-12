use paste::paste;
use seq_macro::seq;

macro_rules! generate_zip_variant {
    ($n:literal) => {
        seq_macro::seq!(N in 1..=$n {
            paste! {
                /// Generates a zip function that combines `$n` indices.
                pub fn [<zip $n>]<In, #( Ix~N, )*>(
                    #( ix~N: Ix~N, )*
                ) -> [<ZipIndex $n>]<In, #( Ix~N, )*>
                where
                    #( Ix~N: crate::Index<In>, )*
                {
                    [<ZipIndex $n>] {
                        #( ix~N, )*
                        _marker: std::marker::PhantomData,
                    }
                }

                /// A struct that represents the index
                pub struct [<ZipIndex $n>]<In, #( Ix~N, )*> {
                    #( ix~N: Ix~N, )*
                    _marker: std::marker::PhantomData<In>,
                }

                /// Implement the Index trait

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

seq!(N in 2..=99 {
    generate_zip_variant!(N);
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Collection;
    use crate::indexes::btree::btree;

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
}
