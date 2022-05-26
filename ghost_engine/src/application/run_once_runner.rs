use super::{Application, ApplicationRunner};

/// This runner executes only one step of the applicaton.
pub struct RunOnceRunner;

impl ApplicationRunner for RunOnceRunner {
    fn run(&mut self, app: &mut Application) {
        app.on_startup();
        app.on_update();
        app.on_shutdown();
    }
}
