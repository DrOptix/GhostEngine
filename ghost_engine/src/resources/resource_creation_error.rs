#[derive(Debug, PartialEq)]
pub enum ResourceCreationError {
    /// Returned when a resource is already registered in the `ResourceManager`.
    AlreadyRegistered,
}
