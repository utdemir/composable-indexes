use composable_indexes::{
    Collection, aggregation,
    index::{
        self, btree::BTreeIndex, grouped::GroupedIndex, hashtable::HashTableIndex,
        premap::PremapIndex, zip::ZipIndex3,
    },
};

type MyIx = ZipIndex3<
    Person,
    PremapIndex<Person, String, HashTableIndex<String>>,
    PremapIndex<Person, u16, BTreeIndex<u16>>,
    GroupedIndex<Person, StarSign, aggregation::CountIndex<Person>>,
>;

struct MyWrapper {
    col: Collection<Person, MyIx>,
}

impl MyWrapper {
    fn new() -> Self {
        Self {
            col: Collection::<Person, MyIx>::new(index::zip!(
                index::premap(|p: &Person| p.name.clone(), index::hashtable()),
                index::premap(|p: &Person| p.birth_year, index::btree()),
                index::grouped(|p: &Person| p.starsign.clone(), || aggregation::count()),
            )),
        }
    }
}

fn main() {
    let mut wrapper = MyWrapper::new();
    wrapper
        .col
        .insert(Person::new("Alice".to_string(), 1990, StarSign::Aries));
    wrapper
        .col
        .insert(Person::new("Bob".to_string(), 1992, StarSign::Gemini));
}

//

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum StarSign {
    Aries,
    Gemini,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Person {
    name: String,
    birth_year: u16,
    starsign: StarSign,
}

impl Person {
    fn new(name: String, birth_year: u16, starsign: StarSign) -> Self {
        Self {
            name,
            birth_year,
            starsign,
        }
    }
}
