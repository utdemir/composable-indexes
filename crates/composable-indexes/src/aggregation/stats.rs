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

/// State for standard deviation calculation using Welford's algorithm.
///
/// Welford's algorithm computes variance and standard deviation incrementally
/// by maintaining:
/// - `mean`: running mean (M in the algorithm)
/// - `sum_sq_diff`: sum of squared differences from the mean (S in the algorithm)
/// - `count`: number of samples (n in the algorithm)
#[derive(Debug, Clone, Copy)]
pub struct StdDevIndexState {
    mean: f64,
    sum_sq_diff: f64,
    count: u64,
}

pub type StdDevIndex<T> = AggregateIndex<T, f64, StdDevIndexState>;

/// Creates a new standard deviation index using Welford's algorithm.
///
/// This index does not hold samples, hence only requiring O(1) space.
///
/// Returns 0.0 when count < 2 (need at least 2 samples for std dev).
///
/// Warning: This implementation is susceptible to numerical instability
/// for very large datasets or values with high variance.
pub fn std_dev<T: Copy + num_traits::ToPrimitive>() -> StdDevIndex<T> {
    AggregateIndex::new(
        StdDevIndexState {
            mean: 0.0,
            sum_sq_diff: 0.0,
            count: 0,
        },
        |st| {
            if st.count < 2 {
                return 0.0;
            }
            // Standard deviation: σ = √(S / (n - 1))
            (st.sum_sq_diff / (st.count - 1) as f64).sqrt()
        },
        |st, op| {
            let x = op.to_f64().unwrap();
            st.count += 1;
            let k = st.count;

            // Adding a sample xₖ:
            // M_new = M_old + (xₖ - M_old) / k
            let old_mean = st.mean;
            st.mean = old_mean + (x - old_mean) / k as f64;

            // S_new = S_old + (xₖ - M_old) * (xₖ - M_new)
            st.sum_sq_diff = st.sum_sq_diff + (x - old_mean) * (x - st.mean);
        },
        |st, op| {
            let x = op.to_f64().unwrap();
            let n = st.count;

            if n <= 1 {
                // Reset to initial state if removing last element
                st.mean = 0.0;
                st.sum_sq_diff = 0.0;
                st.count = 0;
                return;
            }

            // Removing a sample xⱼ:
            // M_new = (n * M_old - xⱼ) / (n - 1)
            let old_mean = st.mean;
            st.mean = (n as f64 * old_mean - x) / (n - 1) as f64;

            // S_new = S_old - (xⱼ - M_old) * (xⱼ - M_new)
            st.sum_sq_diff = st.sum_sq_diff - (x - old_mean) * (x - st.mean);

            // float precision safety: ensure count doesn't go negative
            st.sum_sq_diff = st.sum_sq_diff.max(0.0);

            st.count = n - 1;
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

    #[test]
    fn test_std_dev_basic() {
        use crate::core::Collection;

        let mut db = Collection::new(std_dev::<f64>());

        // Traditional standard deviation calculation that iterates over the collection
        let calculate_std_dev = |collection: &Collection<f64, _>| -> f64 {
            let values: Vec<f64> = collection.iter().into_iter().map(|(_, &v)| v).collect();

            if values.len() < 2 {
                return 0.0;
            }
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance = values
                .iter()
                .map(|&x| {
                    let diff = x - mean;
                    diff * diff
                })
                .sum::<f64>()
                / (values.len() - 1) as f64;
            variance.sqrt()
        };

        // Test with no elements
        assert_eq!(db.query(|ix| ix.get()), calculate_std_dev(&db));

        // Test with one element
        let _k1 = db.insert(5.0);
        assert_eq!(db.query(|ix| ix.get()), calculate_std_dev(&db));

        // Test with two elements: [5.0, 10.0]
        let k2 = db.insert(10.0);
        let expected = calculate_std_dev(&db);
        let result = db.query(|ix| ix.get());
        assert!((result - expected).abs() < 1e-10);

        // Test with three elements: [5.0, 10.0, 15.0]
        let k3 = db.insert(15.0);
        let expected = calculate_std_dev(&db);
        let result = db.query(|ix| ix.get());
        assert!((result - expected).abs() < 1e-10);

        // Remove one element: [5.0, 10.0]
        db.delete_by_key(k3);
        let expected = calculate_std_dev(&db);
        let result = db.query(|ix| ix.get());
        assert!((result - expected).abs() < 1e-10);

        // Remove another: [5.0]
        db.delete_by_key(k2);
        assert_eq!(db.query(|ix| ix.get()), calculate_std_dev(&db));
    }
}
