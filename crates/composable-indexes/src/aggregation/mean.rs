use num_traits::ToPrimitive;

use crate::{
    Index, ShallowClone,
    core::{Insert, Remove, Seal, Update},
};

#[derive(Clone)]
pub struct Mean<T> {
    sum: f64,
    count: u64,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for Mean<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Mean<T> {
    pub fn new() -> Self {
        Mean {
            sum: 0.0,
            count: 0,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Index<T> for Mean<T>
where
    T: ToPrimitive + Copy + 'static,
{
    #[inline]
    fn insert(&mut self, _seal: Seal, op: &Insert<T>) {
        if let Some(val) = op.new.to_f64() {
            self.sum += val;
            self.count += 1;
        }
    }

    #[inline]
    fn remove(&mut self, _seal: Seal, op: &Remove<T>) {
        if let Some(val) = op.existing.to_f64() {
            self.sum -= val;
            self.count -= 1;
        }
    }

    #[inline]
    fn update(&mut self, _seal: Seal, op: &Update<T>) {
        if let (Some(old_val), Some(new_val)) = (op.existing.to_f64(), op.new.to_f64()) {
            self.sum = self.sum - old_val + new_val;
        }
    }
}

impl<T> Mean<T> {
    #[inline]
    pub fn get(&self) -> Option<f64> {
        if self.count > 0 {
            Some(self.sum / self.count as f64)
        } else {
            None
        }
    }
}

impl<T> ShallowClone for Mean<T> where T: Clone {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutils::prop_assert_reference;

    #[test]
    fn test_mean() {
        prop_assert_reference(
            Mean::<u32>::new,
            |db| db.query(|ix| ix.get()),
            |xs| {
                if !xs.is_empty() {
                    let sum: f64 = xs.iter().map(|x| *x as f64).sum();
                    let count = xs.len() as f64;
                    Some(sum / count)
                } else {
                    None
                }
            },
            None,
        );
    }
}
