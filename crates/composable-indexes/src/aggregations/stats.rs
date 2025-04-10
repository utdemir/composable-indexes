use num_traits::Num;
use std::ops::Div;

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

pub struct MeanIndexState<T> {
    sum: T,
    count: u32,
}

pub type MeanIndex<T> = AggregateIndex<T, T, MeanIndexState<T>>;

pub fn mean<T: Num + Copy + Div<u32, Output = T>>() -> MeanIndex<T> {
    AggregateIndex::new(
        MeanIndexState {
            sum: T::zero(),
            count: 0,
        },
        |st| {
            if st.count == 0 {
                return T::zero();
            }
            st.sum / st.count
        },
        |st, op| {
            st.sum = st.sum + *op;
            st.count += 1;
        },
        |st, op| {
            st.sum = st.sum - *op;
            st.count -= 1;
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use composable_indexes_props::prop_assert_reference;
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
}
