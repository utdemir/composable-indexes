use composable_indexes_core::{Index, Insert, Key, Remove, Update};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op<T, Path> {
    Insert(Insert_<T, Path>),
    Update(Update_<T, Path>),
    Remove(Remove_<T, Path>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Insert_<T, Path> {
    pub key: Key<Path>,
    pub new: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update_<T, Path> {
    pub key: Key<Path>,
    pub new: T,
    pub existing: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remove_<T, Path> {
    pub key: Key<Path>,
    pub existing: T,
}

pub fn test_index<T: Clone, Path: Clone>() -> TestIndex<T, Path> {
    TestIndex::new()
}

pub struct TestIndex<T: Clone, Path> {
    pub ops: Vec<Op<T, Path>>,
}

impl<T: Clone, Path> TestIndex<T, Path> {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }
}

impl Default for TestIndex<(), ()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, Path: Clone> Index<T, Path> for TestIndex<T, Path> {
    type Query<'t, Out: 't>
        = TestIndexQueries<'t, T, Out, Path>
    where
        Self: 't,
        Out: 't;

    fn insert(&mut self, op: &Insert<T, Path>) {
        self.ops.push(Op::Insert(Insert_ {
            key: op.key.clone(),
            new: op.new.clone(),
        }));
    }

    fn update(&mut self, op: &Update<T, Path>) {
        self.ops.push(Op::Update(Update_ {
            key: op.key.clone(),
            new: op.new.clone(),
            existing: op.existing.clone(),
        }));
    }

    fn remove(&mut self, op: &Remove<T, Path>) {
        self.ops.push(Op::Remove(Remove_ {
            key: op.key.clone(),
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

pub struct TestIndexQueries<'t, T, Out, Path> {
    pub ops: &'t [Op<T, Path>],
    pub env: composable_indexes_core::QueryEnv<'t, Out>,
}

impl<'t, T: Clone, Out, Path> TestIndexQueries<'t, T, Out, Path> {
    pub fn operations(&self) -> &'t [Op<T, Path>] {
        self.ops
    }
}

#[macro_export]
macro_rules! op_insert {
    ($key:expr, $new:expr) => {
        $crate::Op::Insert($crate::Insert_ {
            key: composable_indexes_core::Key { id: $key, path: () },
            new: $new,
        })
    };
}

#[macro_export]
macro_rules! op_update {
    ($key:expr, $existing:expr, $new:expr) => {
        $crate::Op::Update($crate::Update_ {
            key: composable_indexes_core::Key { id: $key, path: () },
            new: $new,
            existing: $existing,
        })
    };
}

#[macro_export]
macro_rules! op_remove {
    ($key:expr, $existing:expr) => {
        $crate::Op::Remove($crate::Remove_ {
            key: composable_indexes_core::Key { id: $key, path: () },
            existing: $existing,
        })
    };
}
