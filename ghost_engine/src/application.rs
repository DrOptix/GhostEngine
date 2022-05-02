pub struct Application {
    title: String,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            title: "Ghost Engine".to_string(),
        }
    }
}

impl Application {
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn run(&self) {
        println!("Run");
    }
}
