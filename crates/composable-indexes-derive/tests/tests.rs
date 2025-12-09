use composable_indexes::{aggregation, index, Collection};

#[test]
fn zip_to_zip2() {
    let collection = Collection::<u32, _>::new(index::zip!(
        index::btree::<u32>(),
        index::hashtable::<u32>(),
        aggregation::sum::<u32>(),
    ));

    collection.execute(|ix| ix._1().get_one(&1));
    collection.execute(|ix| ix._2().get_one(&1));
    collection.execute(|ix| ix._3().get());
}
