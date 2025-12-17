pub trait ShallowClone: Clone {
    fn shallow_clone(&self) -> Self {
        self.clone()
    }
}
