use downcast_rs::{impl_downcast, Downcast};

/// Gives the ability to store vectors of different types in a hash map
pub trait ComponentBucket: Downcast {
    /// Used to reserve space for a compontent of type `T`. In the storage
    /// the components will be of type `T`, and the fact that an entity has this component or not
    /// will be tracked in [`EntityRecord`].
    fn push_default(&mut self);
}

impl_downcast!(ComponentBucket);

impl<T: Default + 'static> ComponentBucket for Vec<T> {
    fn push_default(&mut self) {
        self.push(T::default());
    }
}
