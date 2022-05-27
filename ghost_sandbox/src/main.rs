use ghost_engine::application::{Application, ApplicationRunner};

use ghost_ecs::EntityId;

#[derive(Default)]
struct Tag {
    pub value: String,
}

#[derive(Default, Clone)]
struct Entities(Vec<EntityId>);

struct InfinityRunner;

impl ApplicationRunner for InfinityRunner {
    fn run(&mut self, app: &mut Application) {
        app.on_startup();

        loop {
            app.on_update();
        }

        // app.on_shutdown();
    }
}

fn main() {
    Application::default()
        .with_title("Sandbox")
        .with_runner(InfinityRunner)
        .with_startup_task(|app| {
            println!("Hello from {}", app.title());

            let _ = app.add_resource::<usize>();
            let _ = app.add_resource::<Entities>();

            let e1 = app.create_entity();
            let e2 = app.create_entity();

            let entities = app.get_resource_mut::<Entities>().unwrap();

            entities.0.push(e1);
            entities.0.push(e2);

            app.add_component_with(e1, || Tag {
                value: "E1".to_string(),
            });

            app.add_component_with(e2, || Tag {
                value: "E2".to_string(),
            });
        })
        .with_update_task(|app| {
            println!("Interation: {}", app.get_resource::<usize>().unwrap());

            let entities = app.get_resource::<Entities>().unwrap();

            for e in entities.0.clone() {
                if let Some(tag) = app.get_component::<Tag>(e) {
                    println!("{} says hello", tag.value);
                }
            }

            if let Some(res) = app.get_resource_mut::<usize>() {
                *res += 1;
            }

            println!();
        })
        .with_shutdown_task(|app| {
            app.remove_resource::<Entities>();

            println!("Bye bye!");
        })
        .run();
}
