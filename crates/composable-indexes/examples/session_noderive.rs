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
type SessionIndex = index::zip::ZipIndex4<
    Session,
    // Index to look up sessions by their session ID
    index::PremapIndex<Session, String, index::HashTableIndex<String>>,
    // Index for range queries on expiration time
    index::PremapOwnedIndex<Session, SystemTime, index::BTreeIndex<SystemTime>>,
    // Grouped index to find all sessions for a given user ID
    index::GroupedIndex<Session, UserId, index::KeysIndex>,
    // Grouped index to count active sessions per country
    index::GroupedIndex<Session, CountryCode, aggregation::CountIndex>,
>;

struct SessionDB {
    db: Collection<Session, SessionIndex>,
}

impl SessionDB {
    fn new() -> Self {
        Self {
            db: Collection::<Session, SessionIndex>::new(index::zip::zip4(
                index::premap(|s: &Session| &s.session_id, index::hashtable()),
                index::premap_owned(|s: &Session| s.expiration_time, index::btree()),
                index::grouped(|s: &Session| s.user_id, || index::keys()),
                index::grouped(|s: &Session| s.country_code, || aggregation::count()),
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
