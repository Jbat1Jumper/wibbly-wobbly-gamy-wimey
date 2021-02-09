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
    scene: Box<dyn Scene>,
    backend: Backend,
    command_bus: Receiver<SceneCommand>,
    command_bus_sender: Sender<SceneCommand>,
}

impl Game {
    fn new(ggez_stuff: (ggez::Context, ggez::event::EventsLoop)) -> GameResult<Game> {
        let (command_bus_sender, command_bus) = channel();
        Ok(Game {
            scene: Box::new(intro::Intro::new()),
            backend: Backend::new(
                ggez_stuff,
                SpriteResources {
                    pyxel_files: map! {
                        "base.pyxel" => {
                            //pyxel::open("resources/base.pyxel")
                            pyxel::load_from_memory(
                                include_bytes!("../resources/base.pyxel")
                            )
                            .expect("Problems loading base.pyxel")
                        }
                    },
                    ..SpriteResources::default()
                },
            )?,
            command_bus,
            command_bus_sender,
        })
    }

    fn run(&mut self) -> GameResult {
        while self.backend.continuing() {
            let button_events = self.backend.poll_events();
            self.handle_button_events(button_events)?;
            self.update()?;
            self.execute_commands_from_bus()?;
            self.draw()?;
        }
        Ok(())
    }

    fn handle_button_events(&mut self, button_events: Vec<(Button, ButtonState)>) -> GameResult {
        for (button, state) in button_events.iter() {
            self.scene.on_input(
                &mut self.backend,
                button,
                state,
                &mut self.command_bus_sender,
            )?
        }
        Ok(())
    }

    fn update(&mut self) -> GameResult {
        self.scene
            .update(&mut self.backend, &mut self.command_bus_sender)
    }

    fn draw(&mut self) -> GameResult {
        self.backend.clear()?;
        self.scene.draw(&mut self.backend)?;
        self.backend.present()?;

        if (self.backend.current_frame() % 100) == 0 {
            println!("FPS: {}", self.backend.get_fps());
        }
        Ok(())
    }

    /// This part could have had more love
    fn create_scene(&mut self, scene_ref: &SceneRef) -> GameResult<Box<dyn Scene>> {
        let scene: Box<dyn Scene> = match scene_ref {
            SceneRef("intro") => Box::new(intro::Intro::new()),
            SceneRef("main_menu") => Box::new(main_menu::MainMenu::new()),
            SceneRef("game_scene") => Box::new(game_scene::GameScene::new()),
            SceneRef(name) => {
                return Err(ggez::GameError::ResourceLoadError(format!(
                    "Error trying to create scene: {}.",
                    name
                )))
            }
        };
        Ok(scene)
    }

    fn execute_commands_from_bus(&mut self) -> GameResult {
        for e in self.command_bus.try_iter().collect::<Vec<_>>().iter() {
            match e {
                SceneCommand::GoTo(scene_ref) => {
                    self.scene = self.create_scene(scene_ref)?;
                }
                SceneCommand::Exit => {
                    self.backend.quit()?;
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
