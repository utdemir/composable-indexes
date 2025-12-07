use crate::{Key, collection::{Insert, Remove, Update}};

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

pub struct Simple<T>(T);

impl<T> QueryResult for Simple<T> {
    type Resolved<U> = T;

    fn map<U, F: Fn(Key) -> U>(self, _f: F) -> Self::Resolved<U> {
        self.0
    }
}

macro_rules! impl_query_result_simple {
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

impl_query_result_simple!(usize, isize);
impl_query_result_simple!(u8, u16, u32, u64, u128);
impl_query_result_simple!(i8, i16, i32, i64, i128);
impl_query_result_simple!(f32, f64);
impl_query_result_simple!(bool);
impl_query_result_simple!(char);
impl_query_result_simple!(String);
impl_query_result_simple!(&'static str);
impl_query_result_simple!(std::num::NonZeroU8, std::num::NonZeroU16, std::num::NonZeroU32, std::num::NonZeroU64, std::num::NonZeroU128);
impl_query_result_simple!(std::num::NonZeroI8, std::num::NonZeroI16, std::num::NonZeroI32, std::num::NonZeroI64, std::num::NonZeroI128);
impl_query_result_simple!(std::num::NonZeroUsize);
impl_query_result_simple!(std::num::NonZeroIsize);

// QueryResult for Array-like types.

impl<T: QueryResult> QueryResult for Option<T> {
    type Resolved<U> = Option<T::Resolved<U>>;

    fn map<U, F: Fn(Key) -> U>(self, f: F) -> Self::Resolved<U> {
        self.map(|v| v.map(f))
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