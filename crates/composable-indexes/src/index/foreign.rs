use crate::{
    Index, ShallowClone,
    core::{Insert, Remove, Seal, Update},
};

impl<T> Index<T> for () {
    #[inline]
    fn insert(&mut self, _seal: Seal, _op: &Insert<T>) {}

    #[inline]
    fn remove(&mut self, _seal: Seal, _op: &Remove<T>) {}

    #[inline]
    fn update(&mut self, _seal: Seal, _op: &Update<T>) {}
}

impl ShallowClone for () {}

impl<T, Inner, const N: usize> Index<T> for [Inner; N]
where
    Inner: Index<T>,
{
    #[inline]
    fn insert(&mut self, seal: Seal, op: &Insert<T>) {
        for inner in self.iter_mut() {
            inner.insert(seal, op);
        }
    }

    #[inline]
    fn remove(&mut self, seal: Seal, op: &Remove<T>) {
        for inner in self.iter_mut() {
            inner.remove(seal, op);
        }
    }

    #[inline]
    fn update(&mut self, seal: Seal, op: &Update<T>) {
        for inner in self.iter_mut() {
            inner.update(seal, op);
        }
    }
}

impl<T, Inner> Index<T> for Vec<Inner>
where
    Inner: Index<T>,
{
    #[inline]
    fn insert(&mut self, seal: Seal, op: &Insert<T>) {
        for inner in self.iter_mut() {
            inner.insert(seal, op);
        }
    }

    #[inline]
    fn remove(&mut self, seal: Seal, op: &Remove<T>) {
        for inner in self.iter_mut() {
            inner.remove(seal, op);
        }
    }

    #[inline]
    fn update(&mut self, seal: Seal, op: &Update<T>) {
        for inner in self.iter_mut() {
            inner.update(seal, op);
        }
    }
}

impl<T, Inner> Index<T> for Option<Inner>
where
    Inner: Index<T>,
{
    #[inline]
    fn insert(&mut self, seal: Seal, op: &Insert<T>) {
        if let Some(inner) = self.as_mut() {
            inner.insert(seal, op);
        }
    }

    #[inline]
    fn remove(&mut self, seal: Seal, op: &Remove<T>) {
        if let Some(inner) = self.as_mut() {
            inner.remove(seal, op);
        }
    }

    #[inline]
    fn update(&mut self, seal: Seal, op: &Update<T>) {
        if let Some(inner) = self.as_mut() {
            inner.update(seal, op);
        }
    }
}

impl<Inner: ShallowClone> ShallowClone for Option<Inner> {}

impl<T, Left, Right> Index<T> for Result<Left, Right>
where
    Left: Index<T>,
    Right: Index<T>,
{
    #[inline]
    fn insert(&mut self, seal: Seal, op: &Insert<T>) {
        match self {
            Ok(left) => left.insert(seal, op),
            Err(right) => right.insert(seal, op),
        }
    }

    #[inline]
    fn remove(&mut self, seal: Seal, op: &Remove<T>) {
        match self {
            Ok(left) => left.remove(seal, op),
            Err(right) => right.remove(seal, op),
        }
    }

    #[inline]
    fn update(&mut self, seal: Seal, op: &Update<T>) {
        match self {
            Ok(left) => left.update(seal, op),
            Err(right) => right.update(seal, op),
        }
    }
}

impl<Left: ShallowClone, Right: ShallowClone> ShallowClone for Result<Left, Right> {}

// Tuples

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

            impl< #( Ix~N: ShallowClone, )* > ShallowClone for ( #( Ix~N, )* ) {}
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
