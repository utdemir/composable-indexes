/*
 * Let's imagine a web application backend that stores in-memory
 * session data for logged-in sessions.
 *
 * It needs to support:
 *
 * - Check if a session ID is valid
 * - Insert new sessions
 * - Expire sessions that are past their expiration time
 * - Being able to "logout from all sessions" for a given user ID
 * - Track the number of active sessions per geolocation for analytics
 *
 * With composable-indexes, we can build an efficient in-memory session store
 * that supports all these operations efficiently without having to manage
 * multiple data structures manually.
 */

#![allow(dead_code)]

use std::time::SystemTime;

use composable_indexes::{Collection, aggregation, index};

#[derive(Clone)]
struct Session {
    session_id: String,
    user_id: UserId,
    expiration_time: SystemTime,
    country_code: CountryCode,
}

#[derive(Clone, composable_indexes::Index, composable_indexes::ShallowClone)]
#[index(Session)]
struct SessionIndex {
    // Index to look up sessions by their session ID
    by_session_id: index::PremapIndex<Session, String, index::im::HashTableIndex<String>>,
    // Index for range queries on expiration time
    by_expiration: index::PremapIndex<Session, SystemTime, index::im::BTreeIndex<SystemTime>>,
    // Grouped index to find all sessions for a given user ID
    by_user_id: index::im::GroupedIndex<Session, UserId, index::im::KeysIndex>,
    // Grouped index to count active sessions per country
    #[index(mark_as_shallow)]
    by_country: index::GroupedIndex<Session, CountryCode, aggregation::CountIndex>,
}

impl SessionIndex {
    fn new() -> Self {
        Self {
            by_session_id: index::premap(
                |s: &Session| s.session_id.clone(),
                index::im::hashtable(),
            ),
            by_expiration: index::premap(|s: &Session| s.expiration_time, index::im::btree()),
            by_user_id: index::im::grouped(|s: &Session| s.user_id, || index::im::keys()),
            by_country: index::grouped(|s: &Session| s.country_code, || aggregation::count()),
        }
    }
}

struct SessionDB {
    db: Collection<Session, SessionIndex>,
}

impl SessionDB {
    fn new() -> Self {
        Self {
            db: Collection::<Session, SessionIndex>::new(SessionIndex::new()),
        }
    }

    fn insert_session(&mut self, session: Session) {
        self.db.insert(session);
    }

    fn get_session(&self, session_id: &String) -> Option<&Session> {
        self.db.query(|ix| ix.by_session_id.get_one(session_id))
    }

    fn delete_expired_sessions(&mut self, now: SystemTime) {
        self.db.delete(|ix| ix.by_expiration.range(..now));
    }

    fn logout_all_sessions(&mut self, user_id: &UserId) {
        self.db
            .delete(|ix| ix.by_user_id.get(user_id).all().collect::<Vec<_>>());
    }

    fn count_sessions_by_country(&self, country_code: &CountryCode) -> u64 {
        self.db.query(|ix| ix.by_country.get(country_code).get())
    }
}

//

fn main() {
    let mut session_db = SessionDB::new();

    session_db.insert_session(Session {
        session_id: "sess1".to_string(),
        user_id: UserId(1),
        expiration_time: SystemTime::now() + std::time::Duration::from_secs(3600),
        country_code: CountryCode::TR,
    });

    session_db.insert_session(Session {
        session_id: "sess2".to_string(),
        user_id: UserId(1),
        expiration_time: SystemTime::now() + std::time::Duration::from_secs(1800),
        country_code: CountryCode::TR,
    });

    session_db.insert_session(Session {
        session_id: "sess3".to_string(),
        user_id: UserId(2),
        expiration_time: SystemTime::now() + std::time::Duration::from_secs(7200),
        country_code: CountryCode::NZ,
    });

    assert!(session_db.count_sessions_by_country(&CountryCode::TR) == 2);
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum CountryCode {
    TR,
    NZ,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct UserId(u32);
