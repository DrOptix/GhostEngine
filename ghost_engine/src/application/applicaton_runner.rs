use downcast_rs::{impl_downcast, Downcast};

use super::Application;

pub trait ApplicationRunner: Downcast {
    fn run(&mut self, app: &mut Application);
}

impl_downcast!(ApplicationRunner);
