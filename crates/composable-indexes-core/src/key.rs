use std::hash::Hash;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Key<Path = ()> {
    pub id: u64,
    pub path: Path,
}

impl<Path> Key<Path> {
    pub fn map_path<NewPath, F>(&self, f: F) -> Key<NewPath>
    where
        F: FnOnce(&Path) -> NewPath,
    {
        Key {
            id: self.id,
            path: f(&self.path),
        }
    }

    pub fn forget_path(&self) -> Key<()> {
        Key {
            id: self.id,
            path: (),
        }
    }
}

impl<Path: Clone> Key<Path> {
    pub fn push<New>(&self, path: New) -> Key<(Path, New)> {
        Key {
            id: self.id,
            path: (self.path.clone(), path),
        }
    }
}
