#![allow(clippy::module_inception)]

use ghost_ecs::{EntityId, Universe};

use crate::resources::{ResourceCreationError, ResourceManager};

use super::{ApplicationRunner, RunOnceRunner};

/// Reperesent a container of logic and data.
///
/// It bundles a `ResourceManager`, a `ghost_ecs::Universe` ECS and necesary
/// startup, shutdown and update actions.
///
/// The actions for a specific stage are executed in the order in wich they are registered.
///
/// The application also can be executed by an `ApplicationRunner` or the `Application`
/// can execute itself with an assigned runner.
///
/// By default an application is constructed with the embedded runner set to
/// `RunOnceRunner`.
/// ```
/// use ghost_engine::application::Application;
///
/// Application::default()
///     .with_startup_task(|_| {
///         println!("Startup task")
///     })
///     .with_shutdown_task(|_| {
///         println!("Shutdown task")
///     })
///     .with_update_task(|_| {
///         println!("Update task")
///     })
///     .run();
/// ```
/// The same can also be expressed with functions instead of closures:
/// ```
/// use ghost_engine::application::Application;
///
/// fn startup(app: &mut Application) {
///     println!("Hello from {}!", app.title());
/// }
///
/// fn shutdown(_: &mut Application) {
///     println!("Shutdown task")
/// }
///
/// fn update(_: &mut Application) {
///     println!("Shutdown task")
/// }
///
/// Application::default()
///     .with_title("Ghost Engine Example")
///     .with_startup_task(|_| {
///     })
///     .with_shutdown_task(|_| {
///         println!("Shutdown task")
///     })
///     .with_update_task(|_| {
///         println!("Update task")
///     })
///     .run();
/// ```
pub struct Application<'app> {
    title: String,

    resources: ResourceManager,
    universe: Universe,

    startup_task: Option<Box<dyn Fn(&mut Application) + 'app>>,
    shutdown_task: Option<Box<dyn Fn(&mut Application) + 'app>>,
    update_task: Option<Box<dyn Fn(&mut Application) + 'app>>,

    runner: Box<dyn ApplicationRunner>,
}

impl Default for Application<'_> {
    fn default() -> Self {
        Self {
            title: "Ghost Engine".to_string(),

            resources: ResourceManager::default(),
            universe: Universe::default(),

            startup_task: None,
            shutdown_task: None,
            update_task: None,

            runner: Box::new(RunOnceRunner),
        }
    }
}

/// Builder methods
impl<'app> Application<'app> {
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn with_runner(mut self, runner: impl ApplicationRunner + 'static) -> Self {
        self.runner = Box::new(runner);
        self
    }

    pub fn with_startup_task(mut self, task: impl Fn(&mut Application) + 'app) -> Self {
        // NOTE: Do we want the same task to be set as both startup and update???
        self.startup_task = Some(Box::new(task));
        self
    }

    pub fn with_shutdown_task(mut self, task: impl Fn(&mut Application) + 'app) -> Self {
        // NOTE: Do we want the same task to be set as both startup and update???
        self.shutdown_task = Some(Box::new(task));
        self
    }

    pub fn with_update_task(mut self, task: impl Fn(&mut Application) + 'app) -> Self {
        // NOTE: Do we want the same task to be set as both startup and update???
        self.update_task = Some(Box::new(task));
        self
    }
}

/// Getters and Setters
impl Application<'_> {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn runner(&self) -> &dyn ApplicationRunner {
        self.runner.as_ref()
    }

    pub fn create_entity(&mut self) -> EntityId {
        self.universe.create_entity()
    }

    pub fn get_resource<T: Default + 'static>(&self) -> Option<&T> {
        self.resources.get_resource()
    }

    pub fn get_resource_mut<T: Default + 'static>(&mut self) -> Option<&mut T> {
        self.resources.get_resource_mut()
    }

    pub fn add_resource<T: Default + 'static>(&mut self) -> Result<(), ResourceCreationError> {
        self.resources.add_resource::<T>()
    }

    pub fn remove_resource<T: Default + 'static>(&mut self) {
        self.resources.remove_resource::<T>();
    }

    pub fn add_component<T: Default + 'static>(&mut self, entity: EntityId) {
        self.universe.add_component::<T>(entity)
    }

    pub fn add_component_with<T, BUILDER>(&mut self, entity: EntityId, builder: BUILDER)
    where
        T: Default + 'static,
        BUILDER: FnOnce() -> T,
    {
        self.universe.add_component_with(entity, builder);
    }

    pub fn get_component<T: Default + 'static>(&self, entity: EntityId) -> Option<&T> {
        self.universe.get_component::<T>(entity)
    }

    pub fn get_component_mut<T: Default + 'static>(&mut self, entity: EntityId) -> Option<&mut T> {
        self.universe.get_component_mut::<T>(entity)
    }
}

/// Life cycle methods
impl Application<'_> {
    pub fn on_startup(&mut self) {
        let mut startup_task = std::mem::take(&mut self.startup_task);

        if let Some(ref mut startup_task) = startup_task {
            startup_task(self);
        }

        self.startup_task = startup_task;
    }

    pub fn on_shutdown(&mut self) {
        let mut shutdown_task = std::mem::take(&mut self.shutdown_task);

        if let Some(ref mut shutdown_task) = shutdown_task {
            shutdown_task(self);
        }

        self.shutdown_task = shutdown_task;
    }

    pub fn on_update(&mut self) {
        let mut update_task = std::mem::take(&mut self.update_task);

        if let Some(ref mut update_task) = update_task {
            update_task(self);
        }

        self.update_task = update_task;
    }

    pub fn run(&mut self) {
        // Take runner outside the context of the application
        // and replace it with a RunOnceRunner.
        let mut runner = std::mem::replace(&mut self.runner, Box::new(RunOnceRunner));

        runner.run(self);

        // We put back the original runner
        self.runner = runner;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_application_with_assigned_custom_runner() {
        struct CustomRunner {
            value: i32,
        }

        impl ApplicationRunner for CustomRunner {
            fn run(&mut self, _: &mut Application) {
                self.value += 1;
            }
        }

        let mut app = Application::default().with_runner(CustomRunner { value: 0 });

        app.run();

        let expected = 1;
        let actual = app
            .runner()
            .as_any()
            .downcast_ref::<CustomRunner>()
            .map_or(-1, |runner| runner.value);

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_run_application_with_custom_runner() {
        struct CustomRunner;

        impl ApplicationRunner for CustomRunner {
            fn run(&mut self, _: &mut Application) {
                // Do nothing
            }
        }

        let mut runner = CustomRunner;
        let mut app = Application::default();

        runner.run(&mut app);
    }
}
