use composable_indexes::{aggregation, index, Collection, ShallowClone as _};
use composable_indexes_derive::{Index, ShallowClone};

#[test]
fn zip_to_zip2() {
    let collection = Collection::<u32, _>::new(index::zip!(
        index::BTree::<u32>::new(),
        index::HashTable::<u32>::new(),
        aggregation::Sum::<u32>::new(),
    ));

    collection.query(|ix| ix._1().get_one(&1));
    collection.query(|ix| ix._2().get_one(&1));
    collection.query(|ix| ix._3().get());
}

#[derive(Clone, ShallowClone)]
struct TestShallowClone {
    field1: index::Trivial,
    field2: aggregation::Count,
}

#[test]
fn test_shallow_clone_derive() {
    let original = TestShallowClone {
        field1: index::Trivial,
        field2: aggregation::Count::new(),
    };

    // Just verify it compiles and executes - the trait implementation is what matters
    let _cloned = original.shallow_clone();
}

#[derive(Clone, Index, ShallowClone)]
#[index(String)]
struct TestBothDerive {
    by_value: index::Trivial,
    count: aggregation::Count,
}

#[test]
fn test_both_derives() {
    let original = TestBothDerive {
        by_value: index::Trivial,
        count: aggregation::Count::new(),
    };

    // Verify both traits work together
    let _cloned = original.shallow_clone();
}

#[derive(Clone, ShallowClone)]
struct TestMarkAsShallow {
    shallow_field: index::Trivial,
    #[index(mark_as_shallow)]
    regular_clone_field: index::Grouped<u32, u32, index::Trivial>,
}

#[test]
fn test_mark_as_shallow() {
    let original = TestMarkAsShallow {
        shallow_field: index::Trivial,
        regular_clone_field: index::Grouped::new(|x: &u32| x, || index::Trivial),
    };

    // Verify the mark_as_shallow attribute works
    let _cloned = original.shallow_clone();
}
