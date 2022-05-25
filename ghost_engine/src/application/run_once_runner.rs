use super::{Application, ApplicationRunner};

pub struct RunOnceRunner;

impl ApplicationRunner for RunOnceRunner {
    fn run(&mut self, app: &mut Application) {
        app.on_startup();
        app.on_update();
        app.on_shutdown();
    }
}
