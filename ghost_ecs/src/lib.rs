type EntityId = usize;

#[derive(Default)]
pub struct Universe {
    next_entity_id: EntityId,
    entities: Vec<EntityId>,
}

impl Universe {
    pub fn create_entity(&mut self) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.push(id);
        id
    }

    pub fn remove_entity(&mut self, entity_id: usize) {
        let index = self.entities.iter().position(|x| *x == entity_id);

        if let Some(index) = index {
            self.entities.remove(index);
        }
    }

    pub fn entity_exists(&self, entity_id: EntityId) -> bool {
        self.entities.contains(&entity_id)
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

        assert!(universe.entity_exists(entity));
    }

    #[test]
    fn can_remove_existing_entity() {
        let mut universe = Universe::default();
        let entity = universe.create_entity();

        universe.remove_entity(entity);

        assert!(!universe.entity_exists(entity));
    }

    #[test]
    fn dont_crash_when_removing_unkown_entity() {
        let mut universe = Universe::default();
        universe.remove_entity(9999);
    }
}
