// use composable_indexes::{
//     indexes::{btree, btree::BTreeQueries, premap},
//     Collection, Index,
// };
// use composable_indexes_core::{Insert, QueryEnv, Remove, Update};

// struct Person {
//     name: String,
//     age: u32,
// }

// // make_index!(Person, {
// //     by_age: premap(|p| p.age, btree()) = BTreeQueries<u32, Person>,
// //     by_name: premap(|p| p.name, btree()) = BTreeQueries<String, Person>,
// // });

// // Translates to:

// fn person_index<'t, Out: 't>() -> impl Index<'t, Person, Query<Out> = PersonQueries<'t, Out>> + 't {
//     SomeIndex {
//         by_age: premap(|p: &Person| p.age, btree()),
//         by_name: premap(|p: &Person| p.name.clone(), btree()),
//     }
// }

// struct PersonQueries<'t, Out> {
//     by_age: BTreeQueries<'t, u32, Out>,
//     by_name: BTreeQueries<'t, String, Out>,
// }

// struct SomeIndex<Ix1, Ix2> {
//     by_age: Ix1,
//     by_name: Ix2,
// }

// impl<'t, Ix1, Ix2> Index<'t, Person> for SomeIndex<Ix1, Ix2>
// where
//     Ix1: Index<'t, Person> + 't,
//     Ix2: Index<'t, Person> + 't,
// {
//     type Query<Out: 't> = PersonQueries<'t, Out>;

//     fn insert(&mut self, op: &Insert<Person>) {
//         self.by_age.insert(op);
//         self.by_name.insert(op);
//     }

//     fn update(&mut self, op: &Update<Person>) {
//         self.by_age.update(op);
//         self.by_name.update(op);
//     }

//     fn remove(&mut self, op: &Remove<Person>) {
//         self.by_age.remove(op);
//         self.by_name.remove(op);
//     }

//     fn query<Out>(&'t self, env: QueryEnv<'t, Out>) -> Self::Query<Out> {
//         PersonQueries {
//             by_age: self.by_age.query(env.clone()),
//             by_name: self.by_name.query(env),
//         }
//     }
// }

// fn main() {
//     let mut collection = Collection::new(person_index());
//     collection.insert(Person {
//         name: "Alice".to_string(),
//         age: 30,
//     });
//     collection.insert(Person {
//         name: "Bob".to_string(),
//         age: 25,
//     });

//     let query = collection.query();

//     // Example usage of the indexes
//     println!("{:?}", query.by_age);
//     println!("{:?}", query.by_name);
// }

fn main() {
    // This is a placeholder for the main function.
    // You can add your test cases or other logic here.
}
