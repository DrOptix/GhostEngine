use std::{any::TypeId, collections::HashMap};

use super::{Resource, ResourceCreationError};

/// The resource manager is a hash map that stores one instance.
#[derive(Default)]
pub struct ResourceManager {
    storage: HashMap<TypeId, Box<dyn Resource>>,
}

/// Builder methods
impl ResourceManager {
    /// Builder method to insert a new resource in the resource manager.1
    /// ```
    /// use ghost_engine::resources::ResourceManager;
    ///
    /// #[derive(Default)]
    /// struct Speed(f32);
    ///
    /// ResourceManager::default().with_resource::<Speed>();
    /// ```
    pub fn with_resource<T: Default + 'static>(mut self) -> Self {
        self.storage
            .insert(TypeId::of::<T>(), Box::new(T::default()));
        self
    }
}

/// Getters and Setters
impl ResourceManager {
    /// Add a resource to an already built `ResourceManager`.
    ///
    /// If the resource was not registered we return `Ok(())`.
    ///
    /// If the resource is already registered with the current `ResourceManager`
    /// instance we
    ///
    /// ## Success example
    /// ```
    /// use ghost_engine::resources::ResourceManager;
    ///
    /// #[derive(Default)]
    /// struct Speed(f32);
    ///
    /// let mut res = ResourceManager::default();
    /// let result = res.add_resource::<Speed>();
    ///
    /// assert_eq!(Ok(()), result);
    /// ```
    ///
    /// ## Error example
    /// ```
    /// use ghost_engine::resources::ResourceManager;
    /// use ghost_engine::resources::ResourceCreationError;
    ///
    /// let mut res = ResourceManager::default();
    /// let _ = res.add_resource::<usize>();
    /// let result = res.add_resource::<usize>();
    ///
    /// assert_eq!(Err(ResourceCreationError::AlreadyRegistered), result);
    /// ```
    pub fn add_resource<T: Default + 'static>(&mut self) -> Result<(), ResourceCreationError> {
        let type_id = TypeId::of::<T>();

        if self.storage.contains_key(&type_id) {
            return Err(ResourceCreationError::AlreadyRegistered);
        }

        self.storage.insert(type_id, Box::new(T::default()));

        Ok(())
    }

    /// Get a const reference to a resource.
    /// Returns `None` if the resource type is not registered with the
    /// `ResourceManager`.
    ///
    /// ```
    /// use ghost_engine::resources::ResourceManager;
    ///
    /// let res = ResourceManager::default().with_resource::<usize>();
    ///
    /// assert_eq!(Some(&usize::default()), res.get_resource::<usize>());
    /// ```
    pub fn get_resource<T: Default + 'static>(&self) -> Option<&T> {
        self.storage
            .get(&TypeId::of::<T>())
            .map(|x| x.as_ref())
            .and_then(|x| x.as_any().downcast_ref::<T>())
    }

    /// Get a mutable reference to a resource.
    /// Returns `None` if the resource type is not registered with the
    /// `ResourceManager`.
    ///
    /// ```
    /// use ghost_engine::resources::ResourceManager;
    ///
    /// #[derive(Debug, Default, PartialEq)]
    /// struct Speed(f32);
    /// let mut res = ResourceManager::default().with_resource::<Speed>();
    ///
    /// if let Some(res) = res.get_resource_mut::<Speed>() {
    ///     res.0 = 1.0;
    /// }
    ///
    /// assert_eq!(Some(&Speed(1.0)), res.get_resource::<Speed>());
    /// ```
    pub fn get_resource_mut<T: Default + 'static>(&mut self) -> Option<&mut T> {
        self.storage
            .get_mut(&TypeId::of::<T>())
            .map(|x| x.as_mut())
            .and_then(|x| x.as_any_mut().downcast_mut::<T>())
    }
}

/// Other methods
impl ResourceManager {
    /// Removes a resource from the `ResourceManager`.
    /// By removing a resource we also free the memory used by the resource.
    ///
    /// ```
    /// use ghost_engine::resources::ResourceManager;
    ///
    /// #[derive(Debug, Default, PartialEq)]
    /// struct Speed(f32);
    ///
    /// let mut res = ResourceManager::default().with_resource::<Speed>();
    /// res.remove_resource::<Speed>();
    ///
    /// assert_eq!(None, res.get_resource::<Speed>());
    /// ```
    pub fn remove_resource<T: Default + 'static>(&mut self) {
        self.storage.remove(&TypeId::of::<T>());
    }
}
