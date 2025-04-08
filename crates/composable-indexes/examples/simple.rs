use composable_indexes::{Collection, aggregations, indexes};

fn main() {
    let mut collection = Collection::<Person, _>::new(indexes::zip3(
        // A hashtable index is useful for fast lookups
        indexes::premap(|p: &Person| p.name.clone(), indexes::hashtable()),
        // A btree index can provide max/min queries
        indexes::premap(|p: &Person| p.birth_year, indexes::btree()),
        // A grouped index can provide a "filtering" view
        indexes::grouped(|p: &Person| p.starsign.clone(), || aggregations::count()),
    ));

    collection.insert(Person::new("Alice".to_string(), 1990, StarSign::Aries));
    collection.insert(Person::new("Bob".to_string(), 1992, StarSign::Taurus));
    collection.insert(Person::new("Charlie".to_string(), 1991, StarSign::Aries));
    collection.insert(Person::new("Dave".to_string(), 1993, StarSign::Gemini));
    let eve = Person::new("Eve".to_string(), 1984, StarSign::Cancer);
    collection.insert(eve.clone());
    collection.insert(Person::new("Frank".to_string(), 1995, StarSign::Gemini));
    collection.insert(Person::new("Grace".to_string(), 1996, StarSign::Cancer));
    collection.insert(Person::new("Heidi".to_string(), 1997, StarSign::Aries));

    let q = collection.query();

    // Find a person by name, using the first index
    let found = q.0.get_one(&"Eve".to_string());
    assert_eq!(found, Some(&eve));

    // Find the youngest person, using the second index
    let youngest = q.1.max_one();
    assert_eq!(
        youngest,
        Some((
            &1997,
            &Person::new("Heidi".to_string(), 1997, StarSign::Aries)
        ))
    );

    // Count the number of Gemini for each star sign, using the third index
    let gemini_count = q.2.get(&StarSign::Gemini);
    assert_eq!(gemini_count, 2);
}

//

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum StarSign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
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
