use paste::paste;
use seq_macro::seq;

#[macro_export]
macro_rules! generate_zip_variant {
    ($n:literal) => {
        seq_macro::seq!(N in 1..=$n {
            paste! {
                /// Generates a zip function that combines `$n` indices.
                pub fn [<zip $n>]<'t, In: 't, #( Ix~N, )*>(
                    #( ix~N: Ix~N, )*
                ) -> [<ZipIndex $n>]<In, #( Ix~N, )*>
                where
                    #( Ix~N: crate::Index<'t, In> + 't, )*
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

                impl<'t, In: 't, #( Ix~N, )*> crate::Index<'t, In> for [<ZipIndex $n>]<In, #( Ix~N, )*>
                where
                    #( Ix~N: crate::Index<'t, In> + 't, )*
                {
                    type Query<Out: 't> = (#(Ix~N::Query<Out>,)*);

                    fn insert(&mut self, op: &crate::Insert<In>) {
                        #(self.ix~N.insert(op);)*
                    }

                    fn update(&mut self, op: &crate::Update<In>) {
                        #(self.ix~N.update(op);)*
                    }

                    fn remove(&mut self, op: &crate::Remove<In>) {
                        #(self.ix~N.remove(op);)*
                    }

                    fn query<Out>(&'t self, env: crate::QueryEnv<'t, Out>) -> Self::Query<Out> {
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

// TODO: Create a zip! macro that takes variadic arguments and calls the appropriate zip function.
// I think this needs to be a proc macro, but that's annoying.

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
