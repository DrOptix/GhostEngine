use downcast_rs::{impl_downcast, Downcast};

use super::Application;

/// Implement this trait if you want to implement a runner or runtime
/// for a `ghost_engine` `Application`.
pub trait ApplicationRunner: Downcast {
    fn run(&mut self, app: &mut Application);
}

impl_downcast!(ApplicationRunner);
