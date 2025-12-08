#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortedVec<T>(Vec<T>);

impl<T: Ord> From<Vec<T>> for SortedVec<T> {
    fn from(mut v: Vec<T>) -> Self {
        v.sort();
        SortedVec(v)
    }
}

impl<T> From<SortedVec<T>> for Vec<T> {
    fn from(sv: SortedVec<T>) -> Self {
        sv.0
    }
}

impl<T: Ord> IntoIterator for SortedVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Ord> FromIterator<T> for SortedVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut v: Vec<T> = iter.into_iter().collect();
        v.sort();
        SortedVec(v)
    }
}