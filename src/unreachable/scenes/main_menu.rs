use glam::f32::Vec2;

use std::env;
use std::path;
use std::time::Duration;

use std::sync::mpsc::{channel, Receiver, Sender};

use std::collections::{HashMap, HashSet};

use legion::*;

use pyxel::Pyxel;

use crate::common::*;

pub struct MainMenu {
    world: World,
}

impl MainMenu {
    pub fn init(world: &mut World, resources: &mut Resources) -> Schedule {
        let font = Font::LiberationMono;

        world.push((
            Text::new("Lost and Found", font, 12),
            Position(Vec2::new(10.0, 10.0)),
        ));

        world.push((
            Text::new("Press space to start!", font, 18),
            Position(Vec2::new(10.0, 25.0)),
        ));

        world.push((
            Text::new("Or press esc to exit", font, 12),
            Position(Vec2::new(10.0, 50.0)),
        ));

        Schedule::builder()
            .add_system(update_main_menu_system())
            .add_system(create_gizmos_system())
            .build()
    }
}

#[system]
fn update_main_menu(
    #[resource] cmd: &mut Vec<SceneCommand>,
    #[resource] input: &Vec<(Button, ButtonState)>,
) {
    for (button, _state) in input.iter() {
        match button {
            Button::A => cmd.push(SceneCommand::GoTo(SceneRef("game"))),
            Button::Start => cmd.push(SceneCommand::Exit),
            _ => (),
        }
    }
}
