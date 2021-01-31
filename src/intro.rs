use ggez;
use glam::f32::Vec2;

use std::env;
use std::path;
use std::time::Duration;

use std::sync::mpsc::{channel, Receiver, Sender};

use std::collections::{HashMap, HashSet};

use legion::*;

use pyxel::Pyxel;

use crate::backend::*;
use crate::common::*;

pub struct Intro {
    remaining: Duration,
}

impl Intro {
    pub fn new() -> Intro {
        Intro {
            remaining: Duration::from_secs(1),
        }
    }
}

impl Scene for Intro {
    fn update(&mut self, ctx: &mut Backend, cmd: &mut Sender<SceneCommand>) -> GameResult {
        let delta = ctx.delta_time();
        if self.remaining > delta {
            self.remaining -= delta;
        } else {
            cmd.send(SceneCommand::GoTo(SceneRef("main_menu")));
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Backend) -> GameResult {
        Ok(())
    }
    fn on_input(&mut self, ctx: &mut Backend, button: &Button, state: &ButtonState, cmd: &mut Sender<SceneCommand>) -> GameResult {
        Ok(())
    }
}
