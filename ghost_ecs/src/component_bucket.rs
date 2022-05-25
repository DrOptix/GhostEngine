use downcast_rs::{impl_downcast, Downcast};

use crate::Index;

pub trait ComponentBucket: Downcast {
    fn push_none(&mut self);
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
