//! This crate implements the ECS. In this case ECS can be seen as both a data
//! structure and a design pattern, with a memory layout like this one:
//!
//!```text
//!                           ┌── EntityRecord(Index) (Occupied or Vacant)
//!                           ▼
//!                         ┌────┐
//!                         │ I1 │     I2      I2          In
//!                       ┌─┼────┼────────────────────────────┐
//!           ┌── T1 ────►│ │ C1 │     C2      C3    ...   Cn │
//!           │           └─┼────┼────────────────────────────┘
//!           │             │    │
//!           │           ┌─┼────┼────────────────────────────┐
//!           │   T2 ────►│ │ C1 │     C2      C3    ...   Cn │
//!  TypeIds ─┤           └─┼────┼────────────────────────────┘
//!           │   ...       │ ...│     ...           ...
//!           │           ┌─┼────┼────────────────────────────┐
//!           └── Tn ────►│ │ C1 │     C2      C3    ...   Cn │
//!                       └─┼────┼────────────────────────────┘
//!                         └────┘
//!                       │                       │
//!                       └───────────┬───────────┘
//!                                   │
//!                     Continous Bucket of Option<Tn>
//!```
//!
//! In words we have one bucket or array, if you will, per each component type.
//!
//! The link between the component type and the bucket where we store it is accomplished by using a hash
//! map to map from a `TypeId` of the component to a memory bucket.
//!
//! In `ghost_ecs` the memory buckets are `Vec<Option<T>>`, where `T` is the component type.
//!
//! When an entity aka [`EntityId`] is created we first check to see if an [`Index`] is [`EntityRecord::Vacant`].
//! To be vacant a record must have all the components pointed by it in all the different buckets set to [`None`].
//! If we don't have a vacant record then we add a new column at the end of each bucket with the coresponding
//! components set to [`None`]. This column at the end will have the record set to [`EntityRecord::Occupied`].
//!
//! The only way to mark a record as vacant is to remove an entity form the universe.
//!
//! As you may already suspect an [`EntityId`] is an incrementing [`usize`]. The link between the real entity id
//! and the real index in the components buckets is kept using a hash map, mapping from [`EntityId`] to
//! [`EntityRecord`].
//!
//! [`EntityRecord`] is just an enum that helps to tag an [`Index`] as either `Occupied` or `Vacant`.
//!
//! ## Performance considerations
//! No real benchmarking was done both memory and CPU wise.
//!
//! On paper this design suffers from the fact that space is wasted by components with a value of [`None`].
//!
//! Even tho the buckets are pieces of continous memory, because the elements can be either [`Some`] or [`None`]
//! we may not get any benefit from vectorization.

mod component_bucket;
mod universe;

pub use component_bucket::*;
pub use universe::*;

/// This represents an entity in `ghost_ecs`.
pub type EntityId = usize;

/// This represents an index of a column in the storage system for `ghost_ecs`.
pub type Index = usize;
