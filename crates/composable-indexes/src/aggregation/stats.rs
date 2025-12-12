//! Common statistical aggregation indexes like count, sum, and mean.
//! These indexes maintain running aggregates that are efficiently updated
//! as elements are added or removed.

use crate::core::{Index, Insert, Remove, Update};
use num_traits::Num;

use super::generic::AggregateIndex;

pub fn count<T: num_traits::Num>() -> CountIndex<T> {
    CountIndex { count: T::zero() }
}

pub struct CountIndex<T = u64> {
    count: T,
}

impl<T, _K> Index<_K> for CountIndex<T>
where
    T: Num + Copy + 'static,
{
    fn insert(&mut self, _op: &Insert<_K>) {
        self.count = self.count + T::one();
    }

    fn remove(&mut self, _op: &Remove<_K>) {
        self.count = self.count - T::one();
    }

    fn update(&mut self, _op: &Update<_K>) {
        // No change in count on update
    }
}

impl<T: Copy> CountIndex<T> {
    pub fn get(&self) -> T {
        self.count
    }
}

pub fn sum<T: Num + Copy>() -> SumIndex<T> {
    AggregateIndex::new(
        T::zero(),
        |st| *st,
        |st, op| *st = *st + *op,
        |st, op| *st = *st - *op,
    )
}

pub type SumIndex<T> = AggregateIndex<T, T, T>;

#[derive(Debug, Clone, Copy)]
pub struct MeanIndexState {
    sum: f64,
    count: u64,
}

pub type MeanIndex<T> = AggregateIndex<T, f64, MeanIndexState>;

pub fn mean<T: Copy + num_traits::ToPrimitive>() -> MeanIndex<T> {
    AggregateIndex::new(
        MeanIndexState { sum: 0., count: 0 },
        |st| {
            if st.count == 0 {
                return 0.;
            }
            st.sum / st.count as f64
        },
        |st, op| {
            st.sum += op.to_f64().unwrap();
            st.count += 1;
        },
        |st, op| {
            st.sum -= op.to_f64().unwrap();
            st.count -= 1;
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutils::prop_assert_reference;
    use core::num::Wrapping;

    #[test]
    fn test_sum() {
        prop_assert_reference(
            || sum::<Wrapping<i16>>(),
            |db| db.query(|ix| ix.get().0),
            |xs| xs.iter().map(|x| Wrapping(x.0)).sum::<Wrapping<i16>>().0,
            None,
        );
    }

    #[test]
    fn test_mean() {
        prop_assert_reference(
            || mean::<u32>(),
            |db| db.query(|ix| ix.get()),
            |xs| {
                if xs.len() > 0 {
                    let sum: f64 = xs.iter().map(|x| *x as f64).sum();
                    let count = xs.len() as f64;
                    sum as f64 / count
                } else {
                    0.
                }
            },
            None,
        );
    }
}
