use crate::{EntityId, Universe};

#[derive(Debug, PartialEq)]
pub enum QueryError {}

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
}

pub struct Query<'q> {
    _universe: &'q mut Universe,
}

impl<'q> Query<'q> {
    pub fn new(universe: &'q mut Universe) -> Self {
        Self {
            _universe: universe,
        }
    }

    pub fn with_component<T>(self) -> Self {
        todo!()
    }

    pub fn run(&mut self) -> Vec<QueryItem> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::Universe;

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

        let query_results = universe.query().with_component::<usize>().run();

        assert_eq!(query_results.len(), 3);

        assert_eq!(query_results[0].entity(), e1);
        assert_eq!(query_results[0].get::<usize>(), Ok(&1_usize));

        assert_eq!(query_results[1].entity(), e6);
        assert_eq!(query_results[1].get::<usize>(), Ok(&6_usize));

        assert_eq!(query_results[0].entity(), e5);
        assert_eq!(query_results[2].get::<usize>(), Ok(&5_usize));
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

        let query_results = universe.query().with_component::<usize>().run();

        assert_eq!(query_results.len(), 3);

        assert_eq!(query_results[0].get_mut::<usize>(), Ok(&mut 1_usize));
        assert_eq!(query_results[1].get_mut::<usize>(), Ok(&mut 6_usize));
        assert_eq!(query_results[2].get_mut::<usize>(), Ok(&mut 5_usize));
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

        let query_results = universe.query().with_component::<usize>().run();

        assert_eq!(query_results.len(), 3);

        assert_eq!(query_results[0].get::<Option<usize>>(), Ok(&Some(1_usize)));
        assert_eq!(query_results[1].get::<Option<usize>>(), Ok(&None));
        assert_eq!(query_results[2].get::<Option<usize>>(), Ok(&Some(6_usize)));
        assert_eq!(query_results[3].get::<Option<usize>>(), Ok(&None));
        assert_eq!(query_results[2].get::<Option<usize>>(), Ok(&Some(5_usize)));
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

        let query_results = universe.query().with_component::<Option<usize>>().run();

        assert_eq!(query_results.len(), 4);

        assert_eq!(
            query_results[0].get_mut::<Option<usize>>(),
            Ok(&mut Some(1_usize))
        );
        assert_eq!(query_results[1].get_mut::<Option<usize>>(), Ok(&mut None));
        assert_eq!(
            query_results[2].get_mut::<Option<usize>>(),
            Ok(&mut Some(6_usize))
        );
        assert_eq!(query_results[3].get_mut::<Option<usize>>(), Ok(&mut None));
        assert_eq!(
            query_results[2].get_mut::<Option<usize>>(),
            Ok(&mut Some(5_usize))
        );
    }
}
