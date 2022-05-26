use downcast_rs::{impl_downcast, Downcast};

use crate::Index;

/// Gives the ability to store vectors of different types in a hash map
pub trait ComponentBucket: Downcast {
    /// Used to reserve space for a compontent of type `T`. In the storage
    /// the components actually have type `Option<T>`. We reserve space for it,
    /// but by default it have a value of `None`.
    fn push_none(&mut self);

    /// Logically remove a component from the bucket by marking it as `None`.
    fn remove_component(&mut self, index: Index);
}

impl_downcast!(ComponentBucket);

impl<T: Default + 'static> ComponentBucket for Vec<Option<T>> {
    fn push_none(&mut self) {
        self.push(None);
    }

    fn remove_component(&mut self, index: Index) {
        self[index] = None;
    }
}
