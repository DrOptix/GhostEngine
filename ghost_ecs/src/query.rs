use std::{any::TypeId, collections::HashMap};

use crate::EntityId;

#[derive(Debug, PartialEq)]
pub enum QueryError {
    UnknownComponent(Vec<String>),
}

#[derive(Debug, PartialEq, Hash, Eq)]
pub enum QueryAccessType {
    Read,
    Write,
    TryRead,
    TryWrite,
}

#[derive(Debug, PartialEq)]
pub struct QueryItem {}

impl QueryItem {
    pub fn entity(&self) -> EntityId {
        todo!()
    }

    pub fn get<T>(&self) -> Result<&T, QueryError> {
        todo!()
    }

    pub fn get_mut<T>(&self) -> Result<&mut T, QueryError> {
        todo!()
    }

    pub fn try_get<T>(&self) -> Result<&Option<T>, QueryError> {
        todo!()
    }

    pub fn try_get_mut<T>(&self) -> Result<&mut Option<T>, QueryError> {
        todo!()
    }
}

pub(crate) struct QueryComponent {
    pub type_id: TypeId,
    pub type_name: String,
}

#[derive(Default)]
pub struct Query {
    components: HashMap<QueryAccessType, Vec<QueryComponent>>,
}

impl Query {
    pub(crate) fn components(&self) -> &HashMap<QueryAccessType, Vec<QueryComponent>> {
        &self.components
    }

    pub fn with_component<T: 'static>(mut self) -> Self {
        self.register_component::<T>(QueryAccessType::Read);
        self
    }

    pub fn with_component_mut<T: 'static>(mut self) -> Self {
        self.register_component::<T>(QueryAccessType::Read);
        self
    }

    pub fn with_optional_component<T: 'static>(mut self) -> Self {
        self.register_component::<T>(QueryAccessType::Read);
        self
    }

    pub fn with_optional_component_mut<T: 'static>(mut self) -> Self {
        self.register_component::<T>(QueryAccessType::Read);
        self
    }

    fn register_component<T: 'static>(&mut self, access_type: QueryAccessType) {
        let type_id = TypeId::of::<T>();

        let query_component = QueryComponent {
            type_id,
            type_name: std::any::type_name::<T>()
                .split("::")
                .last()
                .unwrap()
                .to_string(),
        };

        if !self.components.contains_key(&access_type) {
            self.components
                .insert(QueryAccessType::Read, vec![query_component]);
        } else {
            // SAFETY: at this moment we know for sure we have something
            self.components
                .get_mut(&access_type)
                .unwrap()
                .push(query_component);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Query, QueryError, Universe};

    #[test]
    fn test_query_error_unknown_components() {
        struct Dummy {}

        let mut universe = Universe::default();

        let query = Query::default()
            .with_component::<usize>()
            .with_component_mut::<f32>()
            .with_optional_component::<i32>()
            .with_optional_component_mut::<Dummy>();

        let result = universe.run_query(&query);

        assert_eq!(
            result,
            Err(QueryError::UnknownComponent(vec![
                "usize".to_string(),
                "f32".to_string(),
                "i32".to_string(),
                "Dummy".to_string()
            ]))
        );
    }

    #[test]
    fn test_query_single_read_only_set_component() {
        let mut universe = Universe::default();

        let e1 = universe.create_entity();
        let e2 = universe.create_entity();
        let e3 = universe.create_entity();
        let _ = universe.create_entity();
        let e5 = universe.create_entity();

        // Remove e3 so that e6 will reuse its storage
        universe.remove_entity(e3);

        let e6 = universe.create_entity();

        universe.add_component_with(e1, || 1_usize);
        universe.add_component_with(e6, || 6_usize);
        universe.add_component_with(e5, || 5_usize);

        universe.add_component_with(e1, || 1.0_f32);
        universe.add_component_with(e2, || 2.0_f32);
        universe.add_component_with(e6, || 6.0_f32);
        universe.add_component_with(e5, || 5.0_f32);

        universe.add_component_with(e1, || -1_i32);
        universe.add_component_with(e6, || -6_i32);
        universe.add_component_with(e2, || -2_i32);

        let query = Query::default().with_component::<usize>();
        let query_results = universe.run_query(&query);

        if let Ok(query_results) = query_results {
            assert_eq!(query_results.len(), 3);

            assert_eq!(query_results[0].entity(), e1);
            assert_eq!(query_results[0].get::<usize>(), Ok(&1_usize));

            assert_eq!(query_results[1].entity(), e6);
            assert_eq!(query_results[1].get::<usize>(), Ok(&6_usize));

            assert_eq!(query_results[0].entity(), e5);
            assert_eq!(query_results[2].get::<usize>(), Ok(&5_usize));
        }
    }

    #[test]
    fn test_query_single_mutable_set_component() {
        let mut universe = Universe::default();

        let e1 = universe.create_entity();
        let e2 = universe.create_entity();
        let e3 = universe.create_entity();
        let _ = universe.create_entity();
        let e5 = universe.create_entity();

        // Remove e3 so that e6 will reuse its storage
        universe.remove_entity(e3);

        let e6 = universe.create_entity();

        universe.add_component_with(e1, || 1_usize);
        universe.add_component_with(e6, || 6_usize);
        universe.add_component_with(e5, || 5_usize);

        universe.add_component_with(e1, || 1.0_f32);
        universe.add_component_with(e2, || 2.0_f32);
        universe.add_component_with(e6, || 6.0_f32);
        universe.add_component_with(e5, || 5.0_f32);

        universe.add_component_with(e1, || -1_i32);
        universe.add_component_with(e6, || -6_i32);
        universe.add_component_with(e2, || -2_i32);

        let query = Query::default().with_component_mut::<usize>();
        let query_results = universe.run_query(&query);

        if let Ok(query_results) = query_results {
            assert_eq!(query_results.len(), 3);

            assert_eq!(query_results[0].get_mut::<usize>(), Ok(&mut 1_usize));
            assert_eq!(query_results[1].get_mut::<usize>(), Ok(&mut 6_usize));
            assert_eq!(query_results[2].get_mut::<usize>(), Ok(&mut 5_usize));
        }
    }

    #[test]
    fn test_query_single_read_only_optional_component() {
        let mut universe = Universe::default();

        let e1 = universe.create_entity();
        let e2 = universe.create_entity();
        let e3 = universe.create_entity();
        let _ = universe.create_entity();
        let e5 = universe.create_entity();

        // Remove e3 so that e6 will reuse its storage
        universe.remove_entity(e3);

        let e6 = universe.create_entity();

        universe.add_component_with(e1, || 1_usize);
        universe.add_component_with(e6, || 6_usize);
        universe.add_component_with(e5, || 5_usize);

        universe.add_component_with(e1, || 1.0_f32);
        universe.add_component_with(e2, || 2.0_f32);
        universe.add_component_with(e6, || 6.0_f32);
        universe.add_component_with(e5, || 5.0_f32);

        universe.add_component_with(e1, || -1_i32);
        universe.add_component_with(e6, || -6_i32);
        universe.add_component_with(e2, || -2_i32);

        let query = Query::default().with_optional_component::<usize>();
        let query_results = universe.run_query(&query);

        if let Ok(query_results) = query_results {
            assert_eq!(query_results.len(), 3);

            assert_eq!(query_results[0].try_get::<usize>(), Ok(&Some(1_usize)));
            assert_eq!(query_results[1].try_get::<usize>(), Ok(&None));
            assert_eq!(query_results[2].try_get::<usize>(), Ok(&Some(6_usize)));
            assert_eq!(query_results[3].try_get::<usize>(), Ok(&None));
            assert_eq!(query_results[2].try_get::<usize>(), Ok(&Some(5_usize)));
        }
    }

    #[test]
    fn test_query_single_mutable_optional_component() {
        let mut universe = Universe::default();

        let e1 = universe.create_entity();
        let e2 = universe.create_entity();
        let e3 = universe.create_entity();
        let _ = universe.create_entity();
        let e5 = universe.create_entity();

        // Remove e3 so that e6 will reuse its storage
        universe.remove_entity(e3);

        let e6 = universe.create_entity();

        universe.add_component_with(e1, || 1_usize);
        universe.add_component_with(e6, || 6_usize);
        universe.add_component_with(e5, || 5_usize);

        universe.add_component_with(e1, || 1.0_f32);
        universe.add_component_with(e2, || 2.0_f32);
        universe.add_component_with(e6, || 6.0_f32);
        universe.add_component_with(e5, || 5.0_f32);

        universe.add_component_with(e1, || -1_i32);
        universe.add_component_with(e6, || -6_i32);
        universe.add_component_with(e2, || -2_i32);

        let query = Query::default().with_optional_component_mut::<usize>();
        let query_results = universe.run_query(&query);

        if let Ok(query_results) = query_results {
            assert_eq!(query_results.len(), 4);

            assert_eq!(
                query_results[0].try_get_mut::<usize>(),
                Ok(&mut Some(1_usize))
            );
            assert_eq!(query_results[1].try_get_mut::<usize>(), Ok(&mut None));
            assert_eq!(
                query_results[2].try_get_mut::<usize>(),
                Ok(&mut Some(6_usize))
            );
            assert_eq!(query_results[3].try_get_mut::<usize>(), Ok(&mut None));
            assert_eq!(
                query_results[2].try_get_mut::<usize>(),
                Ok(&mut Some(5_usize))
            );
        }
    }
}
