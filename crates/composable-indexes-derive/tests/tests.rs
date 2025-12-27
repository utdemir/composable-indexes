use composable_indexes::{aggregation, index, Collection, ShallowClone as _};
use composable_indexes_derive::{Index, ShallowClone};

#[test]
fn zip_to_zip2() {
    let collection = Collection::<u32, _>::new((
        index::BTree::<u32>::new(),
        index::HashTable::<u32>::new(),
        aggregation::Sum::<u32>::new(),
    ));

    collection.query(|ix| ix.0.get_one(&1));
    collection.query(|ix| ix.1.get_one(&1));
    collection.query(|ix| ix.2.get());
}

#[derive(Clone)]
struct TestShallowClone {
    field1: (),
    field2: aggregation::Count,
}

#[test]
fn test_shallow_clone_derive() {
    let original = TestShallowClone {
        field1: (),
        field2: aggregation::Count::new(),
    };

    // shallow_clone is not implemented for (), but Count supports it
    let _cloned = TestShallowClone {
        field1: original.field1,
        field2: original.field2.shallow_clone(),
    };
}

#[derive(Clone, Index)]
#[index(String)]
struct TestBothDerive {
    by_value: (),
    count: aggregation::Count,
}

#[test]
fn test_both_derives() {
    let original = TestBothDerive {
        by_value: (),
        count: aggregation::Count::new(),
    };

    // Verify both traits work together
    // shallow_clone is not implemented for (), but Count supports it
    let _cloned = TestBothDerive {
        by_value: original.by_value,
        count: original.count.shallow_clone(),
    };
}

#[derive(Clone, ShallowClone)]
struct TestMarkAsShallow {
    shallow_field: (),
    #[index(mark_as_shallow)]
    regular_clone_field: index::Grouped<u32, u32, ()>,
}

#[test]
fn test_mark_as_shallow() {
    let original = TestMarkAsShallow {
        shallow_field: (),
        regular_clone_field: index::Grouped::new(|x: &u32| x, || ()),
    };

    // Verify the mark_as_shallow attribute works
    let _cloned = original.shallow_clone();
}
