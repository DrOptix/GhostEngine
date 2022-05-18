use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Debug, PartialEq)]
pub enum ResourceCreationError {
    AlreadyRegistered,
}

pub trait ResourceTrait {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Default)]
pub struct Resource<T: Default + 'static> {
    resource: T,
}

impl<T: Default + 'static> ResourceTrait for Resource<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Default)]
pub struct ResourceManager {
    storage: HashMap<TypeId, Box<dyn ResourceTrait>>,
}

/// Builder methods
impl ResourceManager {
    pub fn with_resource<T: Default + 'static>(mut self) -> Self {
        self.storage
            .insert(TypeId::of::<T>(), Box::new(Resource::<T>::default()));
        self
    }
}

/// Getters and Setters
impl ResourceManager {
    pub fn add_resource<T: Default + 'static>(&mut self) -> Result<(), ResourceCreationError> {
        let type_id = TypeId::of::<T>();

        if self.storage.contains_key(&type_id) {
            return Err(ResourceCreationError::AlreadyRegistered);
        }

        self.storage
            .insert(TypeId::of::<T>(), Box::new(Resource::<T>::default()));

        Ok(())
    }

    pub fn get_resource<T: Default + 'static>(&self) -> Option<&T> {
        self.storage
            .get(&TypeId::of::<T>())
            .map(|x| x.as_ref())
            .and_then(|x| x.as_any().downcast_ref::<Resource<T>>())
            .map(|x| &x.resource)
    }

    pub fn get_resource_mut<T: Default + 'static>(&mut self) -> Option<&mut T> {
        self.storage
            .get_mut(&TypeId::of::<T>())
            .map(|x| x.as_mut())
            .and_then(|x| x.as_any_mut().downcast_mut::<Resource<T>>())
            .map(|x| &mut x.resource)
    }
}

/// Other methods
impl ResourceManager {
    pub fn remove_resource<T: Default + 'static>(&mut self) {
        self.storage.remove(&TypeId::of::<T>());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_default_resource_manager() {
        ResourceManager::default();
    }

    #[test]
    fn can_create_resource_manager_with_simple_resource() {
        ResourceManager::default().with_resource::<usize>();
    }

    #[test]
    fn can_create_resource_manager_with_object_resource() {
        #[derive(Default)]
        struct Speed(f32);

        ResourceManager::default().with_resource::<Speed>();
    }

    #[test]
    fn can_add_simple_resource() {
        let mut res = ResourceManager::default();
        let result = res.add_resource::<usize>();

        assert_eq!(Ok(()), result);
    }

    #[test]
    fn can_add_object_resource() {
        #[derive(Default)]

        struct Speed(f32);

        let mut res = ResourceManager::default();
        let result = res.add_resource::<Speed>();

        assert_eq!(Ok(()), result);
    }

    #[test]
    fn can_not_add_same_simple_resource() {
        let mut res = ResourceManager::default();
        let _ = res.add_resource::<usize>();
        let result = res.add_resource::<usize>();

        assert_eq!(Err(ResourceCreationError::AlreadyRegistered), result);
    }

    #[test]
    fn can_not_add_same_object_resource() {
        #[derive(Default)]

        struct Speed(f32);

        let mut res = ResourceManager::default();
        let _ = res.add_resource::<Speed>();
        let result = res.add_resource::<Speed>();

        assert_eq!(Err(ResourceCreationError::AlreadyRegistered), result);
    }

    #[test]
    fn can_get_const_reference_to_simple_resource() {
        let res = ResourceManager::default().with_resource::<usize>();
        assert_eq!(Some(&0), res.get_resource::<usize>());
    }

    #[test]
    fn can_get_const_reference_to_object_resource() {
        #[derive(Debug, Default, PartialEq)]
        struct Speed(f32);
        let res = ResourceManager::default().with_resource::<Speed>();

        assert_eq!(Some(&Speed(0.0)), res.get_resource::<Speed>());
    }

    #[test]
    fn can_get_mut_reference_to_simple_resource() {
        let mut res = ResourceManager::default().with_resource::<usize>();
        assert_eq!(Some(&mut 0), res.get_resource_mut::<usize>());
    }

    #[test]
    fn can_get_mut_reference_to_object_resource() {
        #[derive(Debug, Default, PartialEq)]
        struct Speed(f32);
        let mut res = ResourceManager::default().with_resource::<Speed>();

        assert_eq!(Some(&mut Speed(0.0)), res.get_resource_mut::<Speed>());
    }

    #[test]
    fn can_update_simple_resource() {
        let mut res = ResourceManager::default().with_resource::<usize>();

        res.get_resource_mut::<usize>().and_then(|x| {
            *x = 1;
            Some(x)
        });

        assert_eq!(Some(&1), res.get_resource::<usize>());
    }

    #[test]
    fn can_update_object_resource() {
        #[derive(Debug, Default, PartialEq)]
        struct Speed(f32);

        let mut res = ResourceManager::default().with_resource::<Speed>();

        res.get_resource_mut::<Speed>().and_then(|x| {
            x.0 = 1.0;
            Some(x)
        });

        assert_eq!(Some(&Speed(1.0)), res.get_resource::<Speed>());
    }

    #[test]
    fn can_remove_simple_resource() {
        let mut res = ResourceManager::default().with_resource::<usize>();

        res.remove_resource::<usize>();

        assert_eq!(None, res.get_resource::<usize>());
    }

    #[test]
    fn can_remove_object_resource() {
        #[derive(Debug, Default, PartialEq)]
        struct Speed(f32);

        let mut res = ResourceManager::default().with_resource::<Speed>();
        res.remove_resource::<Speed>();

        assert_eq!(None, res.get_resource::<Speed>());
    }
}
