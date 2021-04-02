use crate::common::*;
use crossbeam_channel::{unbounded, Receiver, Sender};
use legion::{Resources, Schedule, World};

pub trait Plugin {
    fn name(&self) -> String {
        "Unnamed".into()
    }
    fn init(&mut self, world: &mut World, resources: &mut Resources) {}
    fn update(&mut self, world: &mut World, resources: &mut Resources) {}
    fn draw(&mut self, world: &World, resources: &Resources) {}
    fn load_scene(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
        scene: SceneRef,
    ) -> Option<Schedule> {
        None
    }
}

pub struct CurrentSchedule(Schedule);

#[derive(Clone, Debug, Default)]
pub struct PrintProfileEvents(bool);

pub struct Game {
    pub current_scene: String,
    pub scene_schedules: Vec<Option<Schedule>>,
    pub plugins: Vec<Box<dyn Plugin>>,
    pub world: World,
    pub resources: Resources,
}

#[derive(Clone, Debug)]
pub enum ProfileEvent {
    PluginDrawFinished {
        plugin_name: String,
        execution_time: Duration,
    },
    PluginUpdateFinished {
        plugin_name: String,
        execution_time: Duration,
    },
    PluginSceneScheduleFinished {
        plugin_name: String,
        scene_name: String,
        execution_time: Duration,
    },
    FrameFinished {
        execution_time: Duration,
    },
}

impl Game {
    pub fn build() -> Game {
        Game {
            current_scene: "".into(),
            scene_schedules: vec![],
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
        let (profile_event_sender, profile_event_receiver) = unbounded();
        self.resources.insert(profile_event_receiver);

        for plugin in self.plugins.iter_mut() {
            plugin.init(&mut self.world, &mut self.resources);
        }

        self.load_scene(scene);

        loop {
            let frame_started = Instant::now();

            for plugin in self.plugins.iter_mut() {
                let started = Instant::now();
                plugin.update(&mut self.world, &mut self.resources);
                profile_event_sender
                    .send(ProfileEvent::PluginUpdateFinished {
                        plugin_name: plugin.name(),
                        execution_time: started.elapsed(),
                    })
                    .expect("Failed to send profile event");
            }

            for (p, s) in self.plugins.iter().zip(self.scene_schedules.iter_mut()) {
                if let Some(schedule) = s {
                    let started = Instant::now();
                    schedule.execute(&mut self.world, &mut self.resources);
                    profile_event_sender
                        .send(ProfileEvent::PluginSceneScheduleFinished {
                            plugin_name: p.name(),
                            scene_name: self.current_scene.clone(),
                            execution_time: started.elapsed(),
                        })
                        .expect("Failed to send profile event");
                }
            }

            self.execute_commands_from_bus();

            for plugin in self.plugins.iter_mut() {
                let started = Instant::now();
                plugin.draw(&self.world, &self.resources);
                profile_event_sender
                    .send(ProfileEvent::PluginDrawFinished {
                        plugin_name: plugin.name(),
                        execution_time: started.elapsed(),
                    })
                    .expect("Failed to send profile event");
            }

            profile_event_sender
                .send(ProfileEvent::FrameFinished {
                    execution_time: frame_started.elapsed(),
                })
                .expect("Failed to send profile event");

            // We need to drain this so it does not get clogged
            let PrintProfileEvents(print) = *self.resources.get_or_default();
            for p in self.resources.get_mut::<Receiver<ProfileEvent>>().unwrap().try_iter() {
                if print { println!("{:?}", p); }
            }

            if let MustQuit(true) = *self.resources.get_or_default() {
                break;
            }
        }
    }

    fn load_scene(&mut self, scene_ref: SceneRef) {
        // This is harsh and mean
        self.world = World::default();

        self.current_scene = format!("{:?}", scene_ref);
        let mut scene_schedules = vec![];

        for plugin in self.plugins.iter_mut() {
            let s = plugin.load_scene(&mut self.world, &mut self.resources, scene_ref);
            scene_schedules.push(s);
        }
        self.scene_schedules = scene_schedules;
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
