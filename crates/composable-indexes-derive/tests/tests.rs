use composable_indexes::{aggregation, index, Collection};

#[test]
fn zip_to_zip2() {
    let collection = Collection::<u32, _>::new(index::zip!(
        index::btree::<u32>(),
        index::hashtable::<u32>(),
        aggregation::sum::<u32>(),
    ));

    let q = collection.query();
    q.0.get_one(&1);
    q.1.get_one(&1);
    q.2;
}
