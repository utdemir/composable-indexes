/*
 * Variant of 'session.rs' example that demonstrates how to use 'im'-family of indexes.
 */

#![allow(dead_code)]

use std::{rc::Rc, time::SystemTime};

use composable_indexes::{Collection, aggregation, index};

#[derive(Clone)]
struct Session {
    session_id: String,
    user_id: UserId,
    expiration_time: SystemTime,
    country_code: CountryCode,
}

// When used with `im`-family of indexes, deriving `ShallowClone` marks the collection
// as cheap to clone, since the underlying data structures are persistent.
#[derive(Clone, composable_indexes::Index, composable_indexes::ShallowClone)]
// Note: Immutable indexes tend to clone a lot more - so when the payload is large, it makes
// sense to wrap it in an Rc to avoid excessive cloning. This follows 'imbl's own best practices.
#[index(Rc<Session>)]
struct SessionIndex {
    // Most of the time, you just need the 'im' prefix.
    by_session_id: index::PremapIndex<Rc<Session>, String, index::im::HashTableIndex<String>>,
    by_expiration:
        index::PremapOwnedIndex<Rc<Session>, SystemTime, index::im::BTreeIndex<SystemTime>>,
    by_user_id: index::im::GroupedIndex<Rc<Session>, UserId, index::im::KeysIndex>,
    // But sometimes - whether an index is cheap to clone or not cannot be determined by
    // the index alone. For example, the index below is only cheap since `CountryCode` has low
    // cardinality. If it were a high-cardinality key (e.g., first name), it wouldn't be
    // appropriate to mark it as shallow. In those cases, we need to manually mark it.
    // (otherwise, deriving `ShallowClone` would fail to compile)
    #[index(mark_as_shallow)]
    by_country: index::GroupedIndex<Rc<Session>, CountryCode, aggregation::CountIndex>,
}

impl SessionIndex {
    fn new() -> Self {
        Self {
            by_session_id: index::PremapIndex::new(
                |s: &Rc<Session>| &s.session_id,
                index::im::HashTableIndex::new(),
            ),
            by_expiration: index::PremapOwnedIndex::new(
                |s: &Rc<Session>| s.expiration_time,
                index::im::BTreeIndex::<SystemTime>::new(),
            ),
            by_user_id: index::im::GroupedIndex::new(
                |s: &Rc<Session>| s.user_id,
                || index::im::keys(),
            ),
            by_country: index::GroupedIndex::new(
                |s: &Rc<Session>| &s.country_code,
                || aggregation::CountIndex::new(),
            ),
        }
    }
}

#[derive(Clone)]
struct SessionDB {
    db: Collection<Rc<Session>, SessionIndex>,
}

// Immutable indexes mostly have the same API as mutable ones (exceptions being
// a couple of extra Clone bounds here and there).
impl SessionDB {
    fn new() -> Self {
        Self {
            db: Collection::<Rc<Session>, SessionIndex>::new(SessionIndex::new()),
        }
    }

    fn insert_session(&mut self, session: Session) {
        self.db.insert(Rc::new(session));
    }

    fn get_session(&self, session_id: &String) -> Option<&Session> {
        self.db
            .query(|ix| ix.by_session_id.get_one(session_id))
            .map(AsRef::as_ref)
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

    // Now we can create a clone of the database cheaply with structural sharing.
    let mut another_db = session_db.clone();
    another_db.insert_session(Session {
        session_id: "sess4".to_string(),
        user_id: UserId(3),
        expiration_time: SystemTime::now() + std::time::Duration::from_secs(3600),
        country_code: CountryCode::NZ,
    });
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum CountryCode {
    TR,
    NZ,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct UserId(u32);
