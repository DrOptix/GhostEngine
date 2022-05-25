use downcast_rs::{impl_downcast, Downcast};

pub trait Resource: Downcast {}

impl_downcast!(Resource);

impl<T: Default + 'static> Resource for T {}
