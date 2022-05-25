mod component_bucket;
mod universe;

pub use component_bucket::*;
pub use universe::*;

/// This represents an entity in `ghost_ecs`.
pub type EntityId = usize;

/// This represents an index of a column in the storage system for `ghost_ecs`.
pub type Index = usize;
