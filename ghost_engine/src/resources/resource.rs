use downcast_rs::{impl_downcast, Downcast};

/// This trait must be implemented by all structs we want to store
/// as a `ghost_engine` resource.
pub trait Resource: Downcast {}

impl_downcast!(Resource);

/// By default we implement `Resource` for all default constructible types and
/// with a static life time.
impl<T: Default + 'static> Resource for T {}
