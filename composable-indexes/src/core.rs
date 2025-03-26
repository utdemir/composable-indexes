use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    id: u64,
}

pub struct Database<T, Indexes = TrivialIndex> {
    data: HashMap<Key, T>,

    next_key_id: u64,

    indexes: Indexes,
}

impl<T> Database<T> {
    pub fn empty() -> Database<T, TrivialIndex> {
        Database {
            data: HashMap::new(),
            next_key_id: 0,

            indexes: TrivialIndex,
        }
    }
}

impl<'t, T, Indexes> Database<T, Indexes>
where
    Indexes: Index<'t, T, T>,
{
    pub fn insert(&mut self, value: T) -> Key {
        let key = self.mk_key();
        let existing = self.data.insert(key, value);

        // There shouldn't be an existing key, as we just generated it
        debug_assert!(existing.is_none());

        self.indexes.insert(&Insert {
            key,
            new: &self.data[&key],
        });

        key
    }

    pub fn update<F>(&mut self, key: Key, f: F)
    where
        F: FnOnce(Option<&T>) -> T,
    {
        self.alter(key, |existing| Some(f(existing)));
    }

    pub fn delete(&mut self, key: Key) {
        match self.data.remove(&key) {
            Some(ref existing) => {
                self.indexes.remove(&Remove { key, existing });
            }
            None => {}
        }
    }

    pub fn alter<F>(&mut self, key: Key, f: F)
    where
        F: FnOnce(Option<&T>) -> Option<T>,
    {
        match self.data.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let existing = entry.get();
                let new = f(Some(existing));

                match new {
                    Some(new) => {
                        self.indexes.update(&Update {
                            key,
                            new: &new,
                            existing,
                        });
                        entry.insert(new);
                    }
                    None => {
                        self.indexes.remove(&Remove { key, existing });
                        entry.remove();
                    }
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let new = f(None);

                match new {
                    Some(new) => {
                        let key = entry.key().clone();
                        self.indexes.insert(&Insert {
                            key: key,
                            new: &new,
                        });
                        entry.insert(new);
                    }
                    None => {}
                }
            }
        }
    }

    pub fn register_index<NewIndex>(
        self,
        mut index: NewIndex,
    ) -> Database<T, IndexPair<Indexes, NewIndex>>
    where
        NewIndex: Index<'t, T, T>,
    {
        for (key, value) in &self.data {
            index.insert(&Insert {
                key: *key,
                new: value,
            });
        }

        Database {
            data: self.data,
            next_key_id: self.next_key_id,
            indexes: IndexPair {
                l: self.indexes,
                r: index,
            },
        }
    }

    pub fn query(&'t self) -> <Indexes as Index<'t, T, T>>::Queries {
        let env = QueryEnv { data: &self.data };
        self.indexes.query(env)
    }

    fn mk_key(&mut self) -> Key {
        let k = Key {
            id: self.next_key_id,
        };
        self.next_key_id += 1;
        k
    }
}

#[derive(Clone)]
pub struct Insert<'t, T> {
    pub key: Key,
    pub new: &'t T,
}

#[derive(Clone)]
pub struct Update<'t, T> {
    pub key: Key,
    pub new: &'t T,
    pub existing: &'t T,
}

#[derive(Clone)]
pub struct Remove<'t, T> {
    pub key: Key,
    pub existing: &'t T,
}

pub trait Index<'t, In, Out> {
    type Queries;

    fn insert(&mut self, op: &Insert<In>);
    fn update(&mut self, op: &Update<In>);
    fn remove(&mut self, op: &Remove<In>);

    fn query(&'t self, env: QueryEnv<'t, Out>) -> Self::Queries;
}

pub struct QueryEnv<'t, T> {
    pub data: &'t HashMap<Key, T>,
}

impl<'t, T> Clone for QueryEnv<'t, T> {
    fn clone(&self) -> Self {
        QueryEnv { data: self.data }
    }
}

pub struct TrivialIndex;

pub struct TrivialQueries<'t, Out> {
    env: QueryEnv<'t, Out>,
}

impl<Out> TrivialQueries<'_, Out> {
    pub fn get(&self, key: &Key) -> Option<&Out> {
        self.env.data.get(key)
    }
}

impl<'t, In, Out: 't> Index<'t, In, Out> for TrivialIndex {
    type Queries = (TrivialQueries<'t, Out>,);
    fn insert(&mut self, _op: &Insert<In>) {}
    fn update(&mut self, _op: &Update<In>) {}
    fn remove(&mut self, _op: &Remove<In>) {}
    fn query(&'t self, _env: QueryEnv<'t, Out>) -> Self::Queries {
        (TrivialQueries { env: _env },)
    }
}

pub struct IndexPair<Left, Right> {
    pub l: Left,
    pub r: Right,
}

impl<'t, Left, Right, In, Out> Index<'t, In, Out> for IndexPair<Left, Right>
where
    Left: Index<'t, In, Out> + 't,
    Right: Index<'t, In, Out> + 't,
    (
        <Left as Index<'t, In, Out>>::Queries,
        <Right as Index<'t, In, Out>>::Queries,
    ): tupleops::TupleAppend<
            <Left as Index<'t, In, Out>>::Queries,
            <Right as Index<'t, In, Out>>::Queries,
        >,
{
    type Queries = tupleops::Append<Left::Queries, Right::Queries>;

    fn insert(&mut self, op: &Insert<In>) {
        self.l.insert(op);
        self.r.insert(op);
    }

    fn update(&mut self, op: &Update<In>) {
        self.l.update(op);
        self.r.update(op);
    }

    fn remove(&mut self, op: &Remove<In>) {
        self.l.remove(op);
        self.r.remove(op);
    }

    fn query(&'t self, env: QueryEnv<'t, Out>) -> Self::Queries {
        tupleops::append(self.l.query(env.clone()), self.r.query(env))
    }
}
