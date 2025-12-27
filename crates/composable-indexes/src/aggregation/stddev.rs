use num_traits::ToPrimitive;

use crate::{
    Index, ShallowClone,
    core::{Insert, Remove, Seal, Update},
};

/// Standard deviation aggregation index using Welford's algorithm.
///
/// Welford's algorithm computes variance and standard deviation incrementally
/// by maintaining:
/// - `mean`: running mean (M in the algorithm)
/// - `sum_sq_diff`: sum of squared differences from the mean (S in the algorithm)
/// - `count`: number of samples (n in the algorithm)
///
/// This index does not hold samples, thus requiring only O(1) space.
///
/// Returns 0.0 when count < 2 (need at least 2 samples for std dev).
///
/// Warning: This implementation is susceptible to numerical instability
/// for very large datasets or values with high variance.
#[derive(Debug, Clone, Copy)]
pub struct StdDev<T> {
    mean: f64,
    sum_sq_diff: f64,
    count: u64,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for StdDev<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> StdDev<T> {
    pub fn new() -> Self {
        StdDev {
            mean: 0.0,
            sum_sq_diff: 0.0,
            count: 0,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Index<T> for StdDev<T>
where
    T: ToPrimitive + Copy + 'static,
{
    #[inline]
    fn insert(&mut self, _seal: Seal, op: &Insert<T>) {
        if let Some(x) = op.new.to_f64() {
            self.count += 1;
            let k = self.count;

            // Adding a sample xₖ:
            // M_new = M_old + (xₖ - M_old) / k
            let old_mean = self.mean;
            self.mean = old_mean + (x - old_mean) / k as f64;

            // S_new = S_old + (xₖ - M_old) * (xₖ - M_new)
            self.sum_sq_diff += (x - old_mean) * (x - self.mean);
        }
    }

    #[inline]
    fn remove(&mut self, _seal: Seal, op: &Remove<T>) {
        if let Some(x) = op.existing.to_f64() {
            let n = self.count;

            if n <= 1 {
                // Reset to initial state if removing last element
                self.mean = 0.0;
                self.sum_sq_diff = 0.0;
                self.count = 0;
                return;
            }

            // Removing a sample xⱼ:
            // M_new = (n * M_old - xⱼ) / (n - 1)
            let old_mean = self.mean;
            self.mean = (n as f64 * old_mean - x) / (n - 1) as f64;

            // S_new = S_old - (xⱼ - M_old) * (xⱼ - M_new)
            self.sum_sq_diff -= (x - old_mean) * (x - self.mean);

            // float precision safety: ensure count doesn't go negative
            self.sum_sq_diff = self.sum_sq_diff.max(0.0);

            self.count = n - 1;
        }
    }

    #[inline]
    fn update(&mut self, _seal: Seal, op: &Update<T>) {
        if let (Some(old_val), Some(new_val)) = (op.existing.to_f64(), op.new.to_f64()) {
            // For update, we remove the old value and insert the new one
            let n = self.count;

            if n == 0 {
                return;
            }

            if n == 1 {
                // Special case: single element, just update the mean
                self.mean = new_val;
                self.sum_sq_diff = 0.0;
                return;
            }

            // Remove old value
            let old_mean = self.mean;
            let mean_without_old = (n as f64 * old_mean - old_val) / (n - 1) as f64;
            let sum_sq_diff_without_old =
                self.sum_sq_diff - (old_val - old_mean) * (old_val - mean_without_old);

            // Add new value
            let new_mean = mean_without_old + (new_val - mean_without_old) / n as f64;
            let new_sum_sq_diff =
                sum_sq_diff_without_old + (new_val - mean_without_old) * (new_val - new_mean);

            self.mean = new_mean;
            self.sum_sq_diff = new_sum_sq_diff.max(0.0);
        }
    }
}

impl<T> StdDev<T> {
    #[inline]
    pub fn get(&self) -> f64 {
        if self.count < 2 {
            return 0.0;
        }
        // Standard deviation: σ = √(S / (n - 1))
        (self.sum_sq_diff / (self.count - 1) as f64).sqrt()
    }
}

impl<T: Clone> ShallowClone for StdDev<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_std_dev_basic() {
        use crate::core::Collection;

        let mut db = Collection::new(StdDev::<f64>::new());

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
