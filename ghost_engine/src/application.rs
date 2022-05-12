pub trait ApplicationRunner {
    fn run(&mut self, app: &mut Application);

    fn as_any(&self) -> &dyn std::any::Any;
}

pub struct RunOnceRunner;

impl ApplicationRunner for RunOnceRunner {
    fn run(&mut self, app: &mut Application) {
        app.on_startup();
        app.on_update();
        app.on_shutdown();
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Application {
    title: String,
    runner: Box<dyn ApplicationRunner>,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            title: "Ghost Engine".to_string(),
            runner: Box::new(RunOnceRunner),
        }
    }
}

/// Builder methods
impl Application {
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn with_runner(mut self, runner: impl ApplicationRunner + 'static) -> Self {
        self.runner = Box::new(runner);
        self
    }
}

/// Getters
impl Application {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn runner(&self) -> &dyn ApplicationRunner {
        self.runner.as_ref()
    }
}

/// Life cycle methods
impl Application {
    pub fn on_startup(&mut self) {
        // Do Nothing
    }

    pub fn on_shutdown(&mut self) {
        // Do Nothing
    }

    pub fn on_update(&mut self) {
        // Do Nothing
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

            fn as_any(&self) -> &dyn std::any::Any {
                self
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

            fn as_any(&self) -> &dyn std::any::Any {
                self
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

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }

        let mut runner = CustomRunner;
        let mut app = Application::default();

        runner.run(&mut app);
    }
}
