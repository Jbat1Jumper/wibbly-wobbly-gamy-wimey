use ggez;
use glam::f32::Vec2;
use legion::*;
use pyxel::Pyxel;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
#[macro_use]
mod common;
#[macro_use]
use common::*;
mod backend;
mod game_scene;
mod intro;
mod main_menu;
mod pubsub;
use backend::*;

struct Game {
    scene_schedule: Schedule,
    backend: GgezBackend,
    world: World,
    resources: Resources,
}

impl Game {
    fn new(ggez_stuff: (ggez::Context, ggez::event::EventsLoop)) -> GameResult<Game> {
        let mut world = World::default();
        let mut resources = Resources::default();

        resources.insert(PyxelFiles(map! {
            "base.pyxel" => {
                //pyxel::open("resources/base.pyxel")
                pyxel::load_from_memory(
                    include_bytes!("../resources/base.pyxel")
                )
                .expect("Problems loading base.pyxel")
            }
        }));
        let mut backend = GgezBackend::new(ggez_stuff, SpriteResources::default())?;
        backend.init(&mut world, &mut resources);

        let scene_schedule = intro::Intro::init(&mut world, &mut resources);
        Ok(Game {
            backend,
            scene_schedule,
            world,
            resources,
        })
    }

    fn run(&mut self) -> GameResult {
        loop {
            self.backend.update(&mut self.world, &mut self.resources);

            self.scene_schedule
                .execute(&mut self.world, &mut self.resources);

            self.execute_commands_from_bus()?;

            self.backend.draw(&self.world, &self.resources);

            if let MustQuit(true) = *self.resources.get_or_default() {
                break;
            }
        }

        Ok(())
    }

    /// This part could have had more love
    fn load_scene(&mut self, scene_ref: &SceneRef) -> GameResult<()> {
        self.scene_schedule = match scene_ref {
            SceneRef("intro") => intro::Intro::init(&mut self.world, &mut self.resources),
            SceneRef("main_menu") => {
                main_menu::MainMenu::init(&mut self.world, &mut self.resources)
            }
            SceneRef("game_scene") => {
                game_scene::GameScene::init(&mut self.world, &mut self.resources)
            }
            SceneRef(name) => {
                return Err(ggez::GameError::ResourceLoadError(format!(
                    "Error trying to create scene: {}.",
                    name
                )))
            }
        };
        Ok(())
    }

    fn execute_commands_from_bus(&mut self) -> GameResult {
        let command_bus: Vec<_> = {
            let command_bus: &mut Vec<_> = &mut *self.resources.get_mut_or_default();
            command_bus.drain(..).collect()
        };
        for e in command_bus.iter() {
            match e {
                SceneCommand::GoTo(scene_ref) => {
                    self.load_scene(&scene_ref)?;
                }
                SceneCommand::Exit => {
                    self.resources.insert(MustQuit(true));
                }
            }
        }
        Ok(())
    }
}

pub fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("helloworld", "ggez").add_resource_path(resource_dir);
    Game::new(cb.build()?)?.run()
}
