use crate::common::*;
use legion::{World, Resources, Schedule};

pub trait Plugin {
    fn init(&mut self, world: &mut World, resources: &mut Resources) {}
    fn update(&mut self, world: &mut World, resources: &mut Resources) {}
    fn draw(&mut self, world: &World, resources: &Resources) {}
    fn load_scene(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
        scene: SceneRef
    ) -> Option<Schedule> { None }
}

pub struct CurrentSchedule(Schedule);

pub struct Game {
    pub scene_schedule: Schedule,
    pub plugins: Vec<Box<dyn Plugin>>,
    pub world: World,
    pub resources: Resources,
}

impl Game {
    pub fn build() -> Game {
        Game {
            scene_schedule: Schedule::builder().build(),
            plugins: vec![],
            world: World::default(),
            resources: Resources::default(),
        }

    }

    pub fn using<P: 'static + Plugin>(mut self, plugin: P) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    pub fn run(&mut self, scene: SceneRef) {

        for plugin in self.plugins.iter_mut() {
            plugin.init(&mut self.world, &mut self.resources);
        }

        self.load_scene(scene);

        loop {
            for plugin in self.plugins.iter_mut() {
                plugin.update(&mut self.world, &mut self.resources);
            }

            self.scene_schedule
                .execute(&mut self.world, &mut self.resources);

            self.execute_commands_from_bus();

            for plugin in self.plugins.iter_mut() {
                plugin.draw(&self.world, &self.resources);
            }

            if let MustQuit(true) = *self.resources.get_or_default() {
                break;
            }
        }
    }

    fn load_scene(&mut self, scene_ref: SceneRef) {

        // This is harsh and mean
        self.world = World::default();

        let mut steps = vec![];
        for plugin in self.plugins.iter_mut() {
            if let Some(schedule) = plugin.load_scene(&mut self.world, &mut self.resources, scene_ref) {
                steps.append(&mut schedule.into_vec());
            }
        }
        self.scene_schedule = steps.into();
    }

    fn execute_commands_from_bus(&mut self) {
        let command_bus: Vec<_> = {
            let command_bus: &mut Vec<_> = &mut *self.resources.get_mut_or_default();
            command_bus.drain(..).collect()
        };
        for e in command_bus.iter() {
            match e {
                SceneCommand::GoTo(scene_ref) => {
                    self.load_scene(*scene_ref);
                }
                SceneCommand::Exit => {
                    self.resources.insert(MustQuit(true));
                }
            }
        }
    }
}

