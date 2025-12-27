/*
Variant of 'session.rs' example that is implemented without using derive macros.
*/
#![allow(dead_code)]

use std::time::SystemTime;

use composable_indexes::{Collection, aggregation, index};

struct Session {
    session_id: String,
    user_id: UserId,
    expiration_time: SystemTime,
    country_code: CountryCode,
}

// NOTE: Derive macro is completely optional - it's just as easy to use a `composable_indexes::zip::zipN`
// family of combinators to build composite indexes.
type SessionIndex = index::Zip4<
    Session,
    // Index to look up sessions by their session ID
    index::Premap<Session, String, index::HashTable<String>>,
    // Index for range queries on expiration time
    index::PremapOwned<Session, SystemTime, index::BTree<SystemTime>>,
    // Grouped index to find all sessions for a given user ID
    index::Grouped<Session, UserId, index::Keys>,
    // Grouped index to count active sessions per country
    index::Grouped<Session, CountryCode, aggregation::Count>,
>;

struct SessionDB {
    db: Collection<Session, SessionIndex>,
}

impl SessionDB {
    fn new() -> Self {
        Self {
            db: Collection::<Session, SessionIndex>::new(index::zip!(
                index::Premap::new(|s: &Session| &s.session_id, index::HashTable::new()),
                index::PremapOwned::new(
                    |s: &Session| s.expiration_time,
                    index::BTree::<SystemTime>::new(),
                ),
                index::Grouped::new(|s: &Session| &s.user_id, || index::Keys::new()),
                index::Grouped::new(|s: &Session| &s.country_code, || aggregation::Count::new(),),
            )),
        }
    }

    fn insert_session(&mut self, session: Session) {
        self.db.insert(session);
    }

    fn get_session(&self, session_id: &String) -> Option<&Session> {
        self.db.query(|ix| ix._1().get_one(session_id))
    }

    fn delete_expired_sessions(&mut self, now: SystemTime) {
        self.db.delete(|ix| ix._2().range(..now));
    }

    fn logout_all_sessions(&mut self, user_id: &UserId) {
        self.db
            .delete(|ix| ix._3().get(user_id).all().collect::<Vec<_>>());
    }

    fn count_sessions_by_country(&self, country_code: &CountryCode) -> u64 {
        self.db.query(|ix| ix._4().get(country_code).get())
    }
}

//

fn main() {
    // Usage is the same as session.rs example
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum CountryCode {
    TR,
    NZ,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct UserId(u32);
