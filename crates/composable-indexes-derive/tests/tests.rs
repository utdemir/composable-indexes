use composable_indexes::{aggregations, indexes, Collection};

#[test]
fn zip_to_zip2() {
    let collection = Collection::<u32, _>::new(indexes::zip!(
        indexes::btree::<u32>(),
        indexes::hashtable::<u32>(),
        aggregations::sum::<u32>(),
    ));

    let q = collection.query();
    q.0.get_one(&1);
    q.1.get_one(&1);
    q.2;
}
