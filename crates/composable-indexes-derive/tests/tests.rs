use composable_indexes::{aggregation, index, Collection, ShallowClone};
use composable_indexes_derive::{Index, ShallowClone as DeriveShallowClone};

#[test]
fn zip_to_zip2() {
    let collection = Collection::<u32, _>::new(index::zip!(
        index::btree::<u32>(),
        index::hashtable::<u32>(),
        aggregation::sum::<u32>(),
    ));

    collection.query(|ix| ix._1().get_one(&1));
    collection.query(|ix| ix._2().get_one(&1));
    collection.query(|ix| ix._3().get());
}

#[derive(Clone, DeriveShallowClone)]
struct TestShallowClone {
    field1: index::TrivialIndex,
    field2: aggregation::CountIndex,
}

#[test]
fn test_shallow_clone_derive() {
    let original = TestShallowClone {
        field1: index::TrivialIndex,
        field2: aggregation::count(),
    };

    let cloned = original.shallow_clone();

    // Just verify it compiles and executes - the trait implementation is what matters
    drop(cloned);
}

#[derive(Clone, Index, DeriveShallowClone)]
#[index(String)]
struct TestBothDerive {
    by_value: index::TrivialIndex,
    count: aggregation::CountIndex,
}

#[test]
fn test_both_derives() {
    let original = TestBothDerive {
        by_value: index::trivial(),
        count: aggregation::count(),
    };

    let cloned = original.shallow_clone();

    // Verify both traits work together
    drop(cloned);
}
