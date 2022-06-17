use crate::{ComponentBucket, EntityId, Index, Query, QueryAccessType, QueryError, QueryItem};
use bitvec::prelude::*;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// Describes the state of a "column" in the storage system.
///
/// If a column is `Vacant` it means we can use that storage space
/// for a new entity that we will create in the `Universe`.
#[derive(Debug)]
pub enum EntityRecord {
    Occupied { index: Index, component_map: BitVec },
    Vacant(Index),
}

impl EntityRecord {
    pub(crate) fn has_component(&self, component_map_location: usize) -> bool {
        match self {
            EntityRecord::Occupied { component_map, .. } => *component_map
                .get(component_map_location)
                .as_deref()
                .unwrap_or(&false),

            EntityRecord::Vacant(_) => false,
        }
    }
}

/// Stores and exposes operations on entities and components.
///
/// Each entity has a series of components.We have only one instance of a component of each component
/// type per entity.
///
/// The components can be attached, detached, queried and modified.
///
/// More details about the memory model can be found in the crate level documention.
#[derive(Default)]
pub struct Universe {
    next_entity_id: EntityId,
    entity_id_records: HashMap<EntityId, EntityRecord>,
    component_map_locations: HashMap<TypeId, usize>,
    component_buckets: HashMap<TypeId, Box<dyn ComponentBucket>>,
}

impl Universe {
    /// ```
    /// use ghost_ecs::Universe;
    ///
    /// let mut universe = Universe::default();
    /// let entity = universe.create_entity();
    ///
    /// assert_eq!(true, universe.contains_entity(entity));
    /// ```
    pub fn create_entity(&mut self) -> EntityId {
        let new_entity_id = self.next_entity_id;
        let buckets_count = self.component_buckets.keys().len();

        self.next_entity_id += 1;

        let mut component_map = BitVec::with_capacity(buckets_count);
        for _ in 0..buckets_count {
            component_map.push(false);
        }

        let old_entity_id_index = self
            .entity_id_records
            .iter()
            .find_map(|(entity_id, record)| {
                if let EntityRecord::Vacant(index) = record {
                    Some((*entity_id, *index))
                } else {
                    None
                }
            });

        if let Some((old_entity_id, old_entity_index)) = old_entity_id_index {
            self.entity_id_records.insert(
                new_entity_id,
                EntityRecord::Occupied {
                    index: old_entity_index,
                    component_map,
                },
            );

            self.entity_id_records.remove(&old_entity_id);
        } else {
            self.entity_id_records.insert(
                new_entity_id,
                EntityRecord::Occupied {
                    index: buckets_count,
                    component_map,
                },
            );

            for bucket in self.component_buckets.values_mut() {
                bucket.push_default();
            }
        }

        new_entity_id
    }

    /// Removes an entity from `Universe`.
    ///
    /// When an entity is removed the attached components are detached
    /// and marked for reuse by a new entity.
    ///
    /// ```
    /// use ghost_ecs::Universe;
    ///
    /// let mut universe = Universe::default();
    /// let entity = universe.create_entity();
    ///
    /// universe.remove_entity(entity);
    ///
    /// assert_eq!(false, universe.contains_entity(entity));
    /// ```
    pub fn remove_entity(&mut self, entity_id: EntityId) {
        if let Some(record) = self.entity_id_records.get_mut(&entity_id) {
            if let EntityRecord::Occupied {
                index,
                component_map,
            } = record
            {
                for mut bit in component_map.iter_mut() {
                    bit.set(false);
                }

                *record = EntityRecord::Vacant(*index);
            }
        }
    }

    /// Add a component to the entity. The component will be initialized with the default value.
    ///
    /// ```
    /// use ghost_ecs::Universe;
    ///
    /// #[derive(Default)]
    /// struct Component(usize);
    ///
    /// let mut universe = Universe::default();
    /// let entity = universe.create_entity();
    ///
    /// universe.add_component::<Component>(entity);
    ///
    /// assert_eq!(true, universe.has_component::<Component>(entity));
    /// ```
    pub fn add_component<T: Default + 'static>(&mut self, entity_id: EntityId) {
        let type_id = TypeId::of::<T>();

        if let Some(bucket) = self.component_buckets.get_mut(&type_id) {
            if let Some(bucket) = bucket.downcast_mut::<Vec<T>>() {
                if let Some(EntityRecord::Occupied {
                    index,
                    component_map,
                }) = self.entity_id_records.get_mut(&entity_id)
                {
                    let component_map_location =
                        self.component_map_locations.get(&type_id).unwrap();

                    bucket[*index] = T::default();

                    component_map.set(*component_map_location, true);
                }
            }
        } else {
            let capacity = self.entity_id_records.keys().len();
            let mut bucket = Box::new(Vec::<T>::with_capacity(capacity));

            for _ in 0..capacity {
                bucket.push_default();
            }
            self.component_buckets.insert(type_id, bucket);

            let buckets_count = self.component_buckets.len();
            let new_map_location = buckets_count - 1;

            self.component_map_locations
                .insert(type_id, new_map_location);

            for record in self.entity_id_records.values_mut() {
                if let EntityRecord::Occupied { component_map, .. } = record {
                    component_map.push(false);
                }
            }

            if let Some(EntityRecord::Occupied { component_map, .. }) =
                self.entity_id_records.get_mut(&entity_id)
            {
                component_map.set(new_map_location, true);
            }
        }
    }

    /// Add a component to the entity. The component will be initialized with the value built using the `builder` function.
    ///
    /// ```
    /// use ghost_ecs::Universe;
    ///
    /// #[derive(Default, Debug, PartialEq)]
    /// struct Component(usize);
    ///
    /// let mut universe = Universe::default();
    /// let entity = universe.create_entity();
    ///
    /// universe.add_component_with(entity, || { Component(1234) });
    ///
    /// assert_eq!(true, universe.has_component::<Component>(entity));
    /// assert_eq!(Some(&Component(1234)), universe.get_component::<Component>(entity));
    /// ```
    pub fn add_component_with<T, BUILDER>(&mut self, entity: EntityId, builder: BUILDER)
    where
        T: Default + 'static,
        BUILDER: FnOnce() -> T,
    {
        self.add_component::<T>(entity);

        // SAFETY:
        // we just created the component above so it is safe to unwrap.
        let comp = self.get_component_mut::<T>(entity).unwrap();

        *comp = builder();
    }

    /// ```
    /// use ghost_ecs::Universe;
    ///
    /// let mut universe = Universe::default();
    /// let entity = universe.create_entity();
    ///
    /// universe.add_component::<f32>(entity);
    /// universe.remove_component::<f32>(entity);
    ///
    /// assert_eq!(false, universe.has_component::<f32>(entity));
    /// ```
    pub fn remove_component<T: Default + 'static>(&mut self, entity_id: EntityId) {
        let type_id = TypeId::of::<T>();

        if let Some(EntityRecord::Occupied { component_map, .. }) =
            self.entity_id_records.get_mut(&entity_id)
        {
            let map_location = self.component_map_locations.get(&type_id).unwrap();

            component_map.set(*map_location, false);
        }
    }

    /// Check if the universe contains the entity.
    pub fn contains_entity(&self, entity_id: EntityId) -> bool {
        matches!(
            self.entity_id_records.get(&entity_id),
            Some(EntityRecord::Occupied { .. })
        )
    }

    /// Check if a component is attached to an entity.
    pub fn has_component<T: Default + 'static>(&self, entity_id: EntityId) -> bool {
        let type_id = TypeId::of::<T>();

        if let Some(EntityRecord::Occupied { component_map, .. }) =
            self.entity_id_records.get(&entity_id)
        {
            let map_location = self.component_map_locations.get(&type_id).unwrap();

            *component_map
                .get(*map_location)
                .as_deref()
                .unwrap_or(&false)
        } else {
            false
        }
    }

    /// Get a const reference to a component
    ///
    /// ```
    /// use ghost_ecs::Universe;
    ///
    /// let mut universe = Universe::default();
    /// let entity = universe.create_entity();
    //
    /// universe.add_component::<usize>(entity);
    ///
    /// let component = universe.get_component::<usize>(entity);
    ///
    /// assert_eq!(&0, component.unwrap());
    /// ```
    pub fn get_component<T: Default + 'static>(&self, entity_id: EntityId) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let entity_record = self.entity_id_records.get(&entity_id);

        if let Some(EntityRecord::Occupied { index, .. }) = entity_record {
            let bucket = self
                .component_buckets
                .get(&type_id)
                .and_then(|bucket| bucket.downcast_ref::<Vec<T>>());

            if let Some(bucket) = bucket {
                let component = bucket.get(*index);
                return component;
            }
        }

        None
    }

    /// ```
    /// use ghost_ecs::Universe;
    ///
    /// let mut universe = Universe::default();
    /// let entity = universe.create_entity();
    ///
    /// universe.add_component::<usize>(entity);
    ///
    /// let component = universe.get_component_mut::<usize>(entity);
    ///
    /// if let Some(component) = component {
    ///     *component = 1;
    /// }
    ///
    /// let component = universe.get_component::<usize>(entity);
    ///
    /// assert_eq!(&1, component.unwrap());
    /// ```
    pub fn get_component_mut<T: Default + 'static>(
        &mut self,
        entity_id: EntityId,
    ) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let entity_record = self.entity_id_records.get_mut(&entity_id);

        if let Some(EntityRecord::Occupied { index, .. }) = entity_record {
            let bucket = self
                .component_buckets
                .get_mut(&type_id)
                .and_then(|bucket| bucket.downcast_mut::<Vec<Option<T>>>());

            if let Some(bucket) = bucket {
                let component = bucket
                    .get_mut(*index)
                    .and_then(|component| component.as_mut());
                return component;
            }
        }

        None
    }

    pub fn run_query(&mut self, query: &Query) -> Result<Vec<QueryItem>, QueryError> {
        let mut has_errors = false;
        let mut unkown_components = Vec::default();

        for (access_type, query_components) in query.components() {
            for query_component in query_components {
                if !self
                    .component_buckets
                    .contains_key(&query_component.type_id)
                {
                    has_errors = true;
                    unkown_components.push(query_component.type_name.clone());
                } else if !has_errors {
                    for (entity_id, record) in self.entity_id_records.iter_mut() {
                        if let EntityRecord::Occupied { index, .. } = record {
                            todo!()
                            // match access_type {
                            //     QueryAccessType::Read => {
                            //         let x = self
                            //             .component_buckets
                            //             .get(&query_component.type_id)
                            //             .unwrap()
                            //             .downcast::<dyn Any>();
                            //     }

                            //     QueryAccessType::Write => todo!(),
                            //     QueryAccessType::TryRead => todo!(),
                            //     QueryAccessType::TryWrite => todo!(),
                            // }
                        }
                    }
                }
            }
        }

        if !has_errors {
            todo!()
        } else {
            Err(QueryError::UnknownComponent(unkown_components))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_masks_correctly_created() {
        let mut universe = Universe::default();

        let e = universe.create_entity();

        universe.add_component::<usize>(e);
        universe.add_component::<f32>(e);
        universe.add_component::<i32>(e);

        assert_eq!(universe.component_map_locations.keys().len(), 3);

        assert_eq!(
            universe
                .component_map_locations
                .get(&TypeId::of::<usize>())
                .unwrap(),
            &0
        );

        assert_eq!(
            universe
                .component_map_locations
                .get(&TypeId::of::<f32>())
                .unwrap(),
            &1
        );

        assert_eq!(
            universe
                .component_map_locations
                .get(&TypeId::of::<i32>())
                .unwrap(),
            &2
        );
    }

    #[test]
    fn test_entity_record_keeps_track_of_components() {
        let mut universe = Universe::default();

        let e = universe.create_entity();

        universe.add_component::<usize>(e);
        universe.add_component::<f32>(e);
        universe.add_component::<i32>(e);

        let record = universe.entity_id_records.get(&e).unwrap();

        let usize_index = *universe
            .component_map_locations
            .get(&TypeId::of::<usize>())
            .unwrap();

        let f32_index = *universe
            .component_map_locations
            .get(&TypeId::of::<f32>())
            .unwrap();

        let i32_index = *universe
            .component_map_locations
            .get(&TypeId::of::<i32>())
            .unwrap();

        {
            assert!(record.has_component(usize_index));
            assert!(record.has_component(f32_index));
            assert!(record.has_component(i32_index));
        }

        universe.remove_component::<f32>(e);

        {
            let record = universe.entity_id_records.get(&e).unwrap();

            assert!(record.has_component(usize_index));
            assert!(!record.has_component(f32_index));
            assert!(record.has_component(i32_index));
        }
    }

    #[test]
    fn dont_crash_when_removing_unkown_entity() {
        let mut universe = Universe::default();
        universe.remove_entity(9999);
    }

    #[test]
    fn test_add_same_component_to_multiple_entities() {
        let mut universe = Universe::default();
        let entity1 = universe.create_entity();
        let entity2 = universe.create_entity();
        let entity3 = universe.create_entity();

        universe.add_component::<f32>(entity1);
        universe.add_component::<f32>(entity2);
        universe.add_component::<f32>(entity3);

        assert!(universe.has_component::<f32>(entity1));
        assert!(universe.has_component::<f32>(entity2));
        assert!(universe.has_component::<f32>(entity3));
    }

    #[test]
    fn test_add_different_components_to_multiple_entities() {
        let mut universe = Universe::default();
        let entity1 = universe.create_entity();
        let entity2 = universe.create_entity();
        let entity3 = universe.create_entity();

        universe.add_component::<usize>(entity1);
        universe.add_component::<f32>(entity2);
        universe.add_component::<u32>(entity3);

        assert!(universe.has_component::<usize>(entity1));
        assert!(universe.has_component::<f32>(entity2));
        assert!(universe.has_component::<u32>(entity3));
    }

    #[test]
    fn test_add_different_components_to_the_same_entity() {
        let mut universe = Universe::default();
        let entity = universe.create_entity();

        universe.add_component::<usize>(entity);
        universe.add_component::<f32>(entity);

        assert!(universe.has_component::<usize>(entity));
        assert!(universe.has_component::<f32>(entity));
    }

    #[test]
    fn test_keep_memory_integrity_when_removing_non_attached_components() {
        let mut universe = Universe::default();
        let entity1 = universe.create_entity();
        let entity2 = universe.create_entity();

        universe.add_component::<usize>(entity1);
        universe.add_component::<f32>(entity1);

        universe.add_component::<usize>(entity2);
        universe.add_component::<f32>(entity2);
        universe.add_component::<i32>(entity2);

        universe.remove_component::<i32>(entity1);

        assert!(universe.has_component::<usize>(entity1));
        assert!(universe.has_component::<f32>(entity1));

        assert!(universe.has_component::<usize>(entity2));
        assert!(universe.has_component::<f32>(entity2));
        assert!(universe.has_component::<i32>(entity2));
    }

    #[test]
    fn test_reuse_storage_space_from_deleted_entity_and_components() {
        let mut universe = Universe::default();
        let entity1 = universe.create_entity();
        let entity2 = universe.create_entity();
        let entity3 = universe.create_entity();

        universe.add_component::<usize>(entity1);
        universe.add_component::<usize>(entity2);
        universe.add_component::<usize>(entity3);

        universe.add_component::<f32>(entity1);
        universe.add_component::<f32>(entity2);

        universe.add_component::<u32>(entity2);
        universe.add_component::<u32>(entity3);

        universe.remove_entity(entity2);

        let entity4 = universe.create_entity();

        universe.add_component::<f32>(entity4);

        // assert!(!universe.contains_entity(entity2));

        // assert!(universe.has_component::<usize>(entity1));
        // assert!(universe.has_component::<f32>(entity1));
        assert!(!universe.has_component::<u32>(entity1));

        assert!(universe.has_component::<usize>(entity3));
        assert!(!universe.has_component::<f32>(entity3));
        assert!(universe.has_component::<u32>(entity3));

        assert!(!universe.has_component::<usize>(entity4));
        assert!(universe.has_component::<f32>(entity4));
        assert!(!universe.has_component::<u32>(entity4));
    }
}
