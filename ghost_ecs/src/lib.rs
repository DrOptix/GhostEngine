use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
};

type EntityId = usize;
type Index = usize;

#[derive(Debug)]
enum EntityRecord {
    Occupied(Index),
    Vacant(Index),
}

trait ComponentBucketTrait: Debug {
    fn push_none(&mut self);
    fn remove_component(&mut self, index: Index);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Debug + Default + 'static> ComponentBucketTrait for Vec<Option<T>> {
    fn push_none(&mut self) {
        self.push(None);
    }

    fn remove_component(&mut self, index: Index) {
        self[index] = None;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct Universe {
    next_entity_id: EntityId,
    entity_id_records: HashMap<EntityId, EntityRecord>,
    component_buckets: HashMap<TypeId, Box<dyn ComponentBucketTrait>>,
}

impl Universe {
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

    pub fn add_component<T: Debug + Default + 'static>(&mut self, entity_id: EntityId) {
        let type_id = TypeId::of::<T>();
        let capacity = self.entity_id_records.keys().len();

        let entity_record = self.entity_id_records.get_mut(&entity_id);

        if let Some(EntityRecord::Occupied(index)) = entity_record {
            let bucket = self
                .component_buckets
                .get_mut(&type_id)
                .map(|bucket| bucket.as_any_mut())
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

    pub fn remove_component<T: Debug + Default + 'static>(&mut self, entity_id: EntityId) {
        let type_id = TypeId::of::<T>();

        if let Some(bucket) = self.component_buckets.get_mut(&type_id) {
            if let Some(bucket) = bucket.as_any_mut().downcast_mut::<Vec<Option<T>>>() {
                bucket[entity_id] = None;
            }
        }
    }

    pub fn contains_entity(&self, entity_id: EntityId) -> bool {
        matches!(
            self.entity_id_records.get(&entity_id),
            Some(EntityRecord::Occupied(_))
        )
    }

    pub fn has_component<T: Debug + Default + 'static>(&self, entity_id: EntityId) -> bool {
        let type_id = TypeId::of::<T>();

        let bucket = self
            .component_buckets
            .get(&type_id)
            .and_then(|bucket| bucket.as_any().downcast_ref::<Vec<Option<T>>>());

        if let Some(bucket) = bucket {
            if let Some(EntityRecord::Occupied(index)) = self.entity_id_records.get(&entity_id) {
                return bucket[*index].is_some();
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_default_universe() {
        Universe::default();
    }

    #[test]
    fn can_crate_entity() {
        let mut universe = Universe::default();
        let entity = universe.create_entity();

        assert!(universe.contains_entity(entity));
    }

    #[test]
    fn can_remove_existing_entity() {
        let mut universe = Universe::default();
        let entity = universe.create_entity();

        universe.remove_entity(entity);

        assert!(!universe.contains_entity(entity));
    }

    #[test]
    fn dont_crash_when_removing_unkown_entity() {
        let mut universe = Universe::default();
        universe.remove_entity(9999);
    }

    #[test]
    fn can_add_component_to_single_entity() {
        let mut universe = Universe::default();
        let entity = universe.create_entity();

        universe.add_component::<usize>(entity);

        assert!(universe.has_component::<usize>(entity));
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
    fn can_remove_components_from_entity() {
        let mut universe = Universe::default();
        let entity1 = universe.create_entity();
        let entity2 = universe.create_entity();

        universe.add_component::<usize>(entity1);
        universe.add_component::<f32>(entity2);

        universe.remove_component::<usize>(entity1);
        universe.remove_component::<f32>(entity2);

        assert!(!universe.has_component::<usize>(entity1));
        assert!(!universe.has_component::<f32>(entity2));
    }

    #[test]
    fn can_remove_entity_and_attached_components() {
        let mut universe = Universe::default();
        let entity = universe.create_entity();

        universe.add_component::<usize>(entity);
        universe.add_component::<f32>(entity);

        universe.remove_entity(entity);

        assert!(!universe.contains_entity(entity));
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
