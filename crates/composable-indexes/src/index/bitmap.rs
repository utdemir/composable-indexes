use crate::{
    Index,
    core::{Insert, Remove, Seal},
};

pub struct Bitmap {
    data: roaring::RoaringBitmap,
}

impl Bitmap {
    pub fn new() -> Self {
        Bitmap {
            data: roaring::RoaringBitmap::new(),
        }
    }
}

impl Default for Bitmap {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<u32> for Bitmap {
    #[inline]
    fn insert(&mut self, _seal: Seal, op: &Insert<u32>) {
        self.data.insert(*op.new);
    }

    #[inline]
    fn remove(&mut self, _seal: Seal, op: &Remove<u32>) {
        self.data.remove(*op.existing);
    }

    #[inline]
    fn update(&mut self, _seal: Seal, op: &crate::core::Update<u32>) {
        self.data.remove(*op.existing);
        self.data.insert(*op.new);
    }
}

impl Bitmap {
    pub fn contains(&self, value: u32) -> bool {
        self.data.contains(value)
    }

    pub fn contains_range(&self, range: impl core::ops::RangeBounds<u32>) -> bool {
        self.data.contains_range(range)
    }

    pub fn rank(&self, value: u32) -> u64 {
        self.data.rank(value)
    }

    pub fn count_distinct(&self) -> u64 {
        self.data.len()
    }

    pub fn get(&self) -> &roaring::RoaringBitmap {
        &self.data
    }
}

// Treemap

pub struct Treemap {
    data: roaring::RoaringTreemap,
}

impl Treemap {
    pub fn new() -> Self {
        Treemap {
            data: roaring::RoaringTreemap::new(),
        }
    }
}

impl Default for Treemap {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<u64> for Treemap {
    #[inline]
    fn insert(&mut self, _seal: Seal, op: &Insert<u64>) {
        self.data.insert(*op.new);
    }

    #[inline]
    fn remove(&mut self, _seal: Seal, op: &Remove<u64>) {
        self.data.remove(*op.existing);
    }

    #[inline]
    fn update(&mut self, _seal: Seal, op: &crate::core::Update<u64>) {
        self.data.remove(*op.existing);
        self.data.insert(*op.new);
    }
}

impl Treemap {
    pub fn contains(&self, value: u64) -> bool {
        self.data.contains(value)
    }

    pub fn rank(&self, value: u64) -> u64 {
        self.data.rank(value)
    }

    pub fn count_distinct(&self) -> u64 {
        self.data.len()
    }

    pub fn get(&self) -> &roaring::RoaringTreemap {
        &self.data
    }
}
