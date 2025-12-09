use std::num::Wrapping;

use crate::Key;

/// Trait for types that can be returned from queries.
pub trait QueryResult {
    type Resolved<T>;
    fn map<T, F: FnMut(Key) -> T>(self, f: F) -> Self::Resolved<T>;
}

// Sealed marker trait for QueryResults that only return distinct keys.
pub trait QueryResultDistinct: QueryResult {
    #[doc(hidden)]
    fn _seal(_: sealed::Sealed);
}

mod sealed {
    pub struct Sealed;
}

macro_rules! seal {
    ($($t:ty),*) => {
        $(
            impl QueryResultDistinct for $t {
                fn _seal(_: sealed::Sealed) {}
            }
        )*
    };
}

// QueryResult for Key itself.

impl QueryResult for Key {
    type Resolved<T> = T;
    fn map<T, F: FnMut(Key) -> T>(self, mut f: F) -> Self::Resolved<T> {
        f(self)
    }
}
seal!(Key);

// QueryResult for simple types that do not depend on keys.

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Plain<T>(pub T);
impl<T> QueryResult for Plain<T> {
    type Resolved<U> = T;

    fn map<U, F: FnMut(Key) -> U>(self, _f: F) -> Self::Resolved<U> {
        self.0
    }
}

macro_rules! impl_query_result_plain {
    ($($t:ty),*) => {
        $(
            impl QueryResult for $t {
                type Resolved<U> = $t;

                fn map<U, F: FnMut(Key) -> U>(self, _f: F) -> Self::Resolved<U> {
                    self
                }
            }
        )*
    };
}

impl_query_result_plain!(usize, isize);
impl_query_result_plain!(u8, u16, u32, u64, u128);
impl_query_result_plain!(i8, i16, i32, i64, i128);
impl_query_result_plain!(f32, f64);
impl_query_result_plain!(bool);
impl_query_result_plain!(char);
impl_query_result_plain!(String);
impl_query_result_plain!(&'static str);
impl_query_result_plain!(
    std::num::NonZeroU8,
    std::num::NonZeroU16,
    std::num::NonZeroU32,
    std::num::NonZeroU64,
    std::num::NonZeroU128
);
impl_query_result_plain!(
    std::num::NonZeroI8,
    std::num::NonZeroI16,
    std::num::NonZeroI32,
    std::num::NonZeroI64,
    std::num::NonZeroI128
);
impl_query_result_plain!(std::num::NonZeroUsize);
impl_query_result_plain!(std::num::NonZeroIsize);

macro_rules! impl_query_result_plain_wrapper {
    ($t:ident) => {
        impl<T: QueryResult> QueryResult for $t<T> {
            type Resolved<U> = $t<T::Resolved<U>>;

            fn map<U, F: FnMut(Key) -> U>(self, f: F) -> Self::Resolved<U> {
                $t(self.0.map(f))
            }
        }
    };
}

impl_query_result_plain_wrapper!(Wrapping);

// QueryResult for Array-like types.

impl<T: QueryResult> QueryResult for Option<T> {
    type Resolved<U> = Option<T::Resolved<U>>;

    fn map<U, F: FnMut(Key) -> U>(self, f: F) -> Self::Resolved<U> {
        self.map(|v| v.map(f))
    }
}

impl<T: QueryResult, E: QueryResult> QueryResult for Result<T, E> {
    type Resolved<U> = Result<T::Resolved<U>, E::Resolved<U>>;

    fn map<U, F: FnMut(Key) -> U>(self, f: F) -> Self::Resolved<U> {
        match self {
            Ok(v) => Ok(v.map(f)),
            Err(e) => Err(e.map(f)),
        }
    }
}

impl<T: QueryResult> QueryResult for Vec<T> {
    type Resolved<U> = Vec<T::Resolved<U>>;

    fn map<U, F: FnMut(Key) -> U>(self, mut f: F) -> Self::Resolved<U> {
        self.into_iter().map(|v| v.map(&mut f)).collect()
    }
}

impl<T: QueryResult, const N: usize> QueryResult for [T; N] {
    type Resolved<U> = [T::Resolved<U>; N];

    fn map<U, F: FnMut(Key) -> U>(self, mut f: F) -> Self::Resolved<U> {
        self.map(|v| v.map(&mut f))
    }
}

// QueryResult (and QueryResultDistinct) for Set-like types.
impl QueryResult for std::collections::HashSet<Key> {
    type Resolved<T> = Vec<T>;

    fn map<T, F: FnMut(Key) -> T>(self, f: F) -> Self::Resolved<T> {
        self.into_iter().map(f).collect()
    }
}
seal!(std::collections::HashSet<Key>);

impl QueryResult for std::collections::BTreeSet<Key> {
    type Resolved<T> = Vec<T>;

    fn map<T, F: FnMut(Key) -> T>(self, f: F) -> Self::Resolved<T> {
        self.into_iter().map(f).collect()
    }
}
seal!(std::collections::BTreeSet<Key>);

// UnsafeDistinct wrapper

pub struct UnsafeDistinct<T>(pub T);
impl<T: QueryResult> QueryResult for UnsafeDistinct<T> {
    type Resolved<U> = T::Resolved<U>;
    fn map<U, F: FnMut(Key) -> U>(self, f: F) -> Self::Resolved<U> {
        self.0.map(f)
    }
}
impl<T: QueryResultDistinct> QueryResultDistinct for UnsafeDistinct<T> {
    fn _seal(_: sealed::Sealed) {}
}

// QueryResult for tuples

macro_rules! impl_query_result_tuple {
    ($($name:ident),+) => {
        impl<$($name: QueryResult),+> QueryResult for ($($name,)+) {
            type Resolved<U> = ($($name::Resolved<U>,)+);

            fn map<U, F: FnMut(Key) -> U>(self, mut f: F) -> Self::Resolved<U> {
                let ($($name,)+) = self;
                ($($name.map(&mut f),)+)
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
