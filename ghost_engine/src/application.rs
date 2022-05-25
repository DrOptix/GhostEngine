use downcast_rs::{impl_downcast, Downcast};

pub trait ApplicationRunner: Downcast {
    fn run(&mut self, app: &mut Application);
}

impl_downcast!(ApplicationRunner);

pub struct RunOnceRunner;

impl ApplicationRunner for RunOnceRunner {
    fn run(&mut self, app: &mut Application) {
        app.on_startup();
        app.on_update();
        app.on_shutdown();
    }
}

pub struct Application<'app> {
    title: String,

    startup_task: Option<Box<dyn Fn(&mut Application) + 'app>>,
    shutdown_task: Option<Box<dyn Fn(&mut Application) + 'app>>,
    update_task: Option<Box<dyn Fn(&mut Application) + 'app>>,

    runner: Box<dyn ApplicationRunner>,
}

impl Default for Application<'_> {
    fn default() -> Self {
        Self {
            title: "Ghost Engine".to_string(),

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

/// Getters
impl Application<'_> {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn runner(&self) -> &dyn ApplicationRunner {
        self.runner.as_ref()
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
    fn can_create_default_application() {
        Application::default();
    }

    #[test]
    fn can_set_and_get_custom_title() {
        let app = Application::default().with_title("custom");

        let expected = "custom";
        let actual = app.title();

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_set_custom_runner() {
        struct CustomRunner;

        impl ApplicationRunner for CustomRunner {
            fn run(&mut self, _: &mut Application) {
                // Do nothing
            }
        }

        Application::default().with_runner(CustomRunner);
    }

    #[test]
    fn can_run_application_with_assigned_custom_runner() {
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
    fn can_run_application_with_custom_runner() {
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

    #[test]
    fn can_set_function_as_startup_task() {
        fn task(_: &mut Application) {}

        Application::default().with_startup_task(task);
    }

    #[test]
    fn can_set_closure_as_startup_task() {
        let task = |_: &mut Application| {};

        Application::default().with_startup_task(task);
    }

    #[test]
    fn can_execute_closure_as_startup_task() {
        let task = |app: &mut Application| {
            app.title = "Changed".to_string();
        };

        let mut app = Application::default().with_startup_task(task);

        app.run();

        let expected = "Changed";
        let actual = app.title();

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_execute_function_as_startup_task() {
        fn task(app: &mut Application) {
            app.title = "Changed".to_string();
        }

        let mut app = Application::default().with_startup_task(task);
        app.run();

        let expected = "Changed";
        let actual = app.title();

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_set_function_as_update_task() {
        fn task(_: &mut Application) {}

        Application::default().with_update_task(task);
    }

    #[test]
    fn can_set_closure_as_update_task() {
        let task = |_: &mut Application| {};

        Application::default().with_update_task(task);
    }

    #[test]
    fn can_execute_closure_as_update_task() {
        let task = |app: &mut Application| {
            app.title = "Changed".to_string();
        };

        let mut app = Application::default().with_update_task(task);

        app.run();

        let expected = "Changed";
        let actual = app.title();

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_execute_function_as_update_task() {
        fn task(app: &mut Application) {
            app.title = "Changed".to_string();
        }

        let mut app = Application::default().with_update_task(task);
        app.run();

        let expected = "Changed";
        let actual = app.title();

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_set_function_as_shutdown_task() {
        fn task(_: &mut Application) {}

        Application::default().with_shutdown_task(task);
    }

    #[test]
    fn can_set_closure_as_shutdown_task() {
        let task = |_: &mut Application| {};

        Application::default().with_shutdown_task(task);
    }

    #[test]
    fn can_execute_closure_as_shutdown_task() {
        let task = |app: &mut Application| {
            app.title = "Changed".to_string();
        };

        let mut app = Application::default().with_shutdown_task(task);

        app.run();

        let expected = "Changed";
        let actual = app.title();

        assert_eq!(expected, actual);
    }

    #[test]
    fn can_execute_function_as_shutdown_task() {
        fn task(app: &mut Application) {
            app.title = "Changed".to_string();
        }

        let mut app = Application::default().with_shutdown_task(task);
        app.run();

        let expected = "Changed";
        let actual = app.title();

        assert_eq!(expected, actual);
    }
}
