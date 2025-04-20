use composable_indexes_core::{Index, Insert, Key, Remove, Update};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op<T> {
    Insert(Insert_<T>),
    Update(Update_<T>),
    Remove(Remove_<T>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Insert_<T> {
    pub key: Key,
    pub new: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update_<T> {
    pub key: Key,
    pub new: T,
    pub existing: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remove_<T> {
    pub key: Key,
    pub existing: T,
}

pub fn test_index<T: Clone>() -> TestIndex<T> {
    TestIndex::new()
}

pub struct TestIndex<T: Clone> {
    pub ops: Vec<Op<T>>,
}

impl<T: Clone> TestIndex<T> {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }
}

impl Default for TestIndex<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Index<T> for TestIndex<T> {
    type Query<'t, Out: 't>
        = TestIndexQueries<'t, T, Out>
    where
        Self: 't,
        Out: 't;

    fn insert(&mut self, op: &Insert<T>) {
        self.ops.push(Op::Insert(Insert_ {
            key: op.key,
            new: op.new.clone(),
        }));
    }

    fn update(&mut self, op: &Update<T>) {
        self.ops.push(Op::Update(Update_ {
            key: op.key,
            new: op.new.clone(),
            existing: op.existing.clone(),
        }));
    }

    fn remove(&mut self, op: &Remove<T>) {
        self.ops.push(Op::Remove(Remove_ {
            key: op.key,
            existing: op.existing.clone(),
        }));
    }

    fn query<'t, Out: 't>(
        &'t self,
        env: composable_indexes_core::QueryEnv<'t, Out>,
    ) -> Self::Query<'t, Out> {
        TestIndexQueries {
            ops: &self.ops,
            env,
        }
    }
}

pub struct TestIndexQueries<'t, T, Out> {
    pub ops: &'t [Op<T>],
    pub env: composable_indexes_core::QueryEnv<'t, Out>,
}

impl<'t, T: Clone, Out> TestIndexQueries<'t, T, Out> {
    pub fn operations(&self) -> &'t [Op<T>] {
        self.ops
    }
}

#[macro_export]
macro_rules! op_insert {
    ($key:expr, $new:expr) => {
        $crate::Op::Insert($crate::Insert_ {
            key: composable_indexes_core::Key { id: $key },
            new: $new,
        })
    };
}

#[macro_export]
macro_rules! op_update {
    ($key:expr, $existing:expr, $new:expr) => {
        $crate::Op::Update($crate::Update_ {
            key: composable_indexes_core::Key { id: $key },
            new: $new,
            existing: $existing,
        })
    };
}

#[macro_export]
macro_rules! op_remove {
    ($key:expr, $existing:expr) => {
        $crate::Op::Remove($crate::Remove_ {
            key: composable_indexes_core::Key { id: $key },
            existing: $existing,
        })
    };
}
