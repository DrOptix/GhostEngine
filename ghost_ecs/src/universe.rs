use std::{
    any::TypeId,
    collections::{hash_map::Entry, HashMap},
};

use crate::{ComponentBucket, EntityId, Index};

/// Describes the state of a "column" in the storage system.
///
/// If a column is `Vacant` it means we can use that storage space
/// for a new entity that we will create in the `Universe`.
#[derive(Debug)]
pub enum EntityRecord {
    Occupied(Index),
    Vacant(Index),
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

        self.next_entity_id += 1;

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
            self.entity_id_records
                .insert(new_entity_id, EntityRecord::Occupied(old_entity_index));
            self.entity_id_records.remove(&old_entity_id);
        } else {
            let new_entity_index = self.entity_id_records.keys().len();

            self.entity_id_records
                .insert(new_entity_id, EntityRecord::Occupied(new_entity_index));

            for bucket in self.component_buckets.values_mut() {
                bucket.push_none();
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
        let entity_record = self.entity_id_records.get(&entity_id);

        if let Some(&EntityRecord::Occupied(entity_index)) = entity_record {
            for bucket in self.component_buckets.values_mut() {
                bucket.remove_component(entity_index);
            }

            if let Some(entity_record) = self.entity_id_records.get_mut(&entity_id) {
                *entity_record = EntityRecord::Vacant(entity_index)
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
        let capacity = self.entity_id_records.keys().len();

        let entity_record = self.entity_id_records.get_mut(&entity_id);

        if let Some(EntityRecord::Occupied(index)) = entity_record {
            let bucket = self
                .component_buckets
                .get_mut(&type_id)
                .and_then(|bucket| bucket.downcast_mut::<Vec<Option<T>>>());

            if let Some(bucket) = bucket {
                bucket[*index] = Some(T::default());
            }
        }

        if let Entry::Vacant(entry) = self.component_buckets.entry(type_id) {
            let mut bucket = Box::new(Vec::<Option<T>>::with_capacity(capacity));

            for _ in 0..capacity {
                bucket.push_none();
            }

            bucket[entity_id] = Some(T::default());

            entry.insert(bucket);
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

        if let Some(bucket) = self.component_buckets.get_mut(&type_id) {
            if let Some(bucket) = bucket.downcast_mut::<Vec<Option<T>>>() {
                bucket[entity_id] = None;
            }
        }
    }

    /// Check if the universe contains the entity.
    pub fn contains_entity(&self, entity_id: EntityId) -> bool {
        matches!(
            self.entity_id_records.get(&entity_id),
            Some(EntityRecord::Occupied(_))
        )
    }

    /// Check if a component is attached to an entity.
    pub fn has_component<T: Default + 'static>(&self, entity_id: EntityId) -> bool {
        let type_id = TypeId::of::<T>();

        let bucket = self
            .component_buckets
            .get(&type_id)
            .and_then(|bucket| bucket.downcast_ref::<Vec<Option<T>>>());

        if let Some(bucket) = bucket {
            if let Some(EntityRecord::Occupied(index)) = self.entity_id_records.get(&entity_id) {
                return bucket[*index].is_some();
            }
        }

        false
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

        if let Some(EntityRecord::Occupied(index)) = entity_record {
            let bucket = self
                .component_buckets
                .get(&type_id)
                .and_then(|bucket| bucket.downcast_ref::<Vec<Option<T>>>());

            if let Some(bucket) = bucket {
                let component = bucket.get(*index).and_then(|component| component.as_ref());
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

        if let Some(EntityRecord::Occupied(index)) = entity_record {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dont_crash_when_removing_unkown_entity() {
        let mut universe = Universe::default();
        universe.remove_entity(9999);
    }

    #[test]
    fn can_add_same_component_to_multiple_entities() {
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
    fn can_add_different_components_to_multiple_entities() {
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
    fn can_add_different_components_to_the_same_entity() {
        let mut universe = Universe::default();
        let entity = universe.create_entity();

        universe.add_component::<usize>(entity);
        universe.add_component::<f32>(entity);

        assert!(universe.has_component::<usize>(entity));
        assert!(universe.has_component::<f32>(entity));
    }

    #[test]
    fn can_keep_memory_integrity_when_removing_non_attached_components() {
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
    fn can_reuse_storage_space_from_deleted_entity_and_components() {
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

        assert!(!universe.contains_entity(entity2));

        assert!(universe.has_component::<usize>(entity1));
        assert!(universe.has_component::<f32>(entity1));
        assert!(!universe.has_component::<u32>(entity1));

        assert!(universe.has_component::<usize>(entity3));
        assert!(!universe.has_component::<f32>(entity3));
        assert!(universe.has_component::<u32>(entity3));

        assert!(!universe.has_component::<usize>(entity4));
        assert!(universe.has_component::<f32>(entity4));
        assert!(!universe.has_component::<u32>(entity4));
    }
}
