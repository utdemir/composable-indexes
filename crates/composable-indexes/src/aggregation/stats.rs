//! Common statistical aggregation indexes like count, sum, and mean.
//! These indexes maintain running aggregates that are efficiently updated
//! as elements are added or removed.

use num_traits::Num;

use super::generic::AggregateIndex;

pub fn count<T>() -> AggregateIndex<T, u32, u32> {
    AggregateIndex::new(0, |st| *st, |st, _op| *st += 1, |st, _op| *st -= 1)
}

pub type CountIndex<T> = AggregateIndex<T, u32, u32>;

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
    count: u32,
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
            st.sum = st.sum + op.to_f64().unwrap();
            st.count += 1;
        },
        |st, op| {
            st.sum = st.sum - op.to_f64().unwrap();
            st.count -= 1;
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use composable_indexes_testutils::prop_assert_reference;
    use std::num::Wrapping;

    #[test]
    fn test_sum() {
        prop_assert_reference(
            || sum::<Wrapping<i16>>(),
            |q| *q,
            |xs| xs.iter().sum(),
            None,
        );
    }

    #[test]
    fn test_mean() {
        prop_assert_reference(
            || mean::<u32>(),
            |q| *q,
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
