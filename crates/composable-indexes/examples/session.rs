/*
 * Let's imagine a web application backend that (for some reason) stores in-memory
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

struct Session {
    session_id: String,
    user_id: UserId,
    expiration_time: SystemTime,
    country_code: CountryCode,
}

type SessionIndex = index::zip::ZipIndex4<
    Session,
    index::PremapIndex<Session, String, index::HashTableIndex<String>>,
    index::PremapIndex<Session, SystemTime, index::BTreeIndex<SystemTime>>,
    index::GroupedIndex<Session, UserId, index::KeysIndex>,
    index::GroupedIndex<Session, CountryCode, aggregation::CountIndex>,
>;

struct SessionDB {
    db: Collection<Session, SessionIndex>,
}

impl SessionDB {
    fn new() -> Self {
        Self {
            db: Collection::<Session, SessionIndex>::new(index::zip!(
                index::premap(|s: &Session| s.session_id.clone(), index::hashtable()),
                index::premap(|s: &Session| s.expiration_time, index::btree()),
                index::grouped(|s: &Session| s.user_id, || index::keys()),
                index::grouped(|s: &Session| s.country_code, || aggregation::count()),
            )),
        }
    }

    fn insert_session(&mut self, session: Session) {
        self.db.insert(session);
    }

    fn get_session(&self, session_id: &String) -> Option<&Session> {
        self.db.query(|ix| ix._1().inner().get_one(session_id))
    }

    fn delete_expired_sessions(&mut self, now: SystemTime) {
        self.db.delete(|ix| ix._2().inner().range(..now));
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

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
enum CountryCode {
    TR,
    NZ,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
struct UserId(u32);
