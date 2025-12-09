use std::num::Wrapping;

use crate::{
    collection::{Insert, Remove, Update},
    Key,
};

/// Trait of indexes. You probably only need this if you're implementing a new index.
pub trait Index<In> {
    #[doc(hidden)]
    fn insert(&mut self, op: &Insert<In>);

    #[doc(hidden)]
    fn remove(&mut self, op: &Remove<In>);

    #[doc(hidden)]
    fn update(&mut self, op: &Update<In>) {
        self.remove(&Remove {
            key: op.key,
            existing: op.existing,
        });
        self.insert(&Insert {
            key: op.key,
            new: op.new,
        });
    }
}

pub trait QueryResult {
    type Resolved<T>;
    fn map<T, F: Fn(Key) -> T>(self, f: F) -> Self::Resolved<T>;
}

impl QueryResult for Key {
    type Resolved<T> = T;
    fn map<T, F: Fn(Key) -> T>(self, f: F) -> Self::Resolved<T> {
        f(self)
    }
}

// QueryResult for simple types that do not depend on keys.

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Identity<T>(pub T);

impl<T> QueryResult for Identity<T> {
    type Resolved<U> = T;

    fn map<U, F: Fn(Key) -> U>(self, _f: F) -> Self::Resolved<U> {
        self.0
    }
}

macro_rules! impl_query_result_identity {
    ($($t:ty),*) => {
        $(
            impl QueryResult for $t {
                type Resolved<U> = $t;

                fn map<U, F: Fn(Key) -> U>(self, _f: F) -> Self::Resolved<U> {
                    self
                }
            }
        )*
    };
}

impl_query_result_identity!(usize, isize);
impl_query_result_identity!(u8, u16, u32, u64, u128);
impl_query_result_identity!(i8, i16, i32, i64, i128);
impl_query_result_identity!(f32, f64);
impl_query_result_identity!(bool);
impl_query_result_identity!(char);
impl_query_result_identity!(String);
impl_query_result_identity!(&'static str);
impl_query_result_identity!(
    std::num::NonZeroU8,
    std::num::NonZeroU16,
    std::num::NonZeroU32,
    std::num::NonZeroU64,
    std::num::NonZeroU128
);
impl_query_result_identity!(
    std::num::NonZeroI8,
    std::num::NonZeroI16,
    std::num::NonZeroI32,
    std::num::NonZeroI64,
    std::num::NonZeroI128
);
impl_query_result_identity!(std::num::NonZeroUsize);
impl_query_result_identity!(std::num::NonZeroIsize);

macro_rules! impl_query_result_identity_wrapper {
    ($t:ident) => {
        impl<T: QueryResult> QueryResult for $t<T> {
            type Resolved<U> = $t<T::Resolved<U>>;

            fn map<U, F: Fn(Key) -> U>(self, f: F) -> Self::Resolved<U> {
                $t(self.0.map(f))
            }
        }
    };
}

impl_query_result_identity_wrapper!(Wrapping);

// QueryResult for Array-like types.

impl<T: QueryResult> QueryResult for Option<T> {
    type Resolved<U> = Option<T::Resolved<U>>;

    fn map<U, F: Fn(Key) -> U>(self, f: F) -> Self::Resolved<U> {
        self.map(|v| v.map(f))
    }
}

impl<T: QueryResult, E: QueryResult> QueryResult for Result<T, E> {
    type Resolved<U> = Result<T::Resolved<U>, E::Resolved<U>>;

    fn map<U, F: Fn(Key) -> U>(self, f: F) -> Self::Resolved<U> {
        match self {
            Ok(v) => Ok(v.map(f)),
            Err(e) => Err(e.map(f)),
        }
    }
}

impl<T: QueryResult> QueryResult for Vec<T> {
    type Resolved<U> = Vec<T::Resolved<U>>;

    fn map<U, F: Fn(Key) -> U>(self, f: F) -> Self::Resolved<U> {
        self.into_iter().map(|v| v.map(&f)).collect()
    }
}

impl<T: QueryResult, const N: usize> QueryResult for [T; N] {
    type Resolved<U> = [T::Resolved<U>; N];

    fn map<U, F: Fn(Key) -> U>(self, f: F) -> Self::Resolved<U> {
        self.map(|v| v.map(&f))
    }
}

// QueryResult for tuples

macro_rules! impl_query_result_tuple {
    ($($name:ident),+) => {
        impl<$($name: QueryResult),+> QueryResult for ($($name,)+) {
            type Resolved<U> = ($($name::Resolved<U>,)+);

            fn map<U, F: Fn(Key) -> U>(self, f: F) -> Self::Resolved<U> {
                let ($($name,)+) = self;
                ($($name.map(&f),)+)
            }
        }
    };
}

impl_query_result_tuple!(_1);
impl_query_result_tuple!(_1, _2);
impl_query_result_tuple!(_1, _2, _3);
impl_query_result_tuple!(_1, _2, _3, _4);
impl_query_result_tuple!(_1, _2, _3, _4, _5);
impl_query_result_tuple!(_1, _2, _3, _4, _5, _6);
impl_query_result_tuple!(_1, _2, _3, _4, _5, _6, _7);
impl_query_result_tuple!(_1, _2, _3, _4, _5, _6, _7, _8);
impl_query_result_tuple!(_1, _2, _3, _4, _5, _6, _7, _8, _9);
impl_query_result_tuple!(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10);
impl_query_result_tuple!(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11);
impl_query_result_tuple!(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12);
impl_query_result_tuple!(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13);
impl_query_result_tuple!(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14);
impl_query_result_tuple!(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15);
impl_query_result_tuple!(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16);
