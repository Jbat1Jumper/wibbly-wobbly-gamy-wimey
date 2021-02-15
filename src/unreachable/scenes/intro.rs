use ggez;
use glam::f32::Vec2;

use std::env;
use std::path;
use std::time::Duration;

use std::sync::mpsc::{channel, Receiver, Sender};

use std::collections::{HashMap, HashSet};

use legion::*;

use pyxel::Pyxel;

use crate::common::*;

pub struct Intro {
    remaining: Duration,
}

impl Intro {
    pub fn init(world: &mut World, resources: &mut Resources) -> Schedule {
        resources.insert(RemainingIntroTime(Duration::from_secs(1)));

        let font = Font::LiberationMono;
        world.push((
            Text::new("SOGA", font, 32),
            Position(Vec2::new(90.0, 90.0)),
        ));


        Schedule::builder()
            .add_system(update_intro_system())
            .build()
    }
}

struct RemainingIntroTime(Duration);

#[system]
fn update_intro(
    #[resource] LastFrameDuration(delta): &LastFrameDuration,
    #[resource] RemainingIntroTime(remaining): &mut RemainingIntroTime,
    #[resource] cmd: &mut Vec<SceneCommand>,
) {
    if *remaining > *delta {
        *remaining -= *delta;
    } else {
        cmd.push(SceneCommand::GoTo(SceneRef("main_menu")));
    }
}
