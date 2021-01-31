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


pub struct MainMenu {
    world: World,
}

impl MainMenu {
    pub fn new() -> MainMenu {
        let mut m = MainMenu {
            world: World::default(),
        };
        m.init();
        m
    }
    fn init(&mut self) {
        let font = Font::LiberationMono;
        self.world.push((
            Text::new("Lost and Found", font, 48),
            Position(Vec2::new(10.0, 20.0)),
        ));

        self.world.push((
            Text::new("Press space to start!", font, 40),
            Position(Vec2::new(10.0, 80.0)),
        ));

        self.world.push((
            Text::new("Or press esc to exit", font, 40),
            Position(Vec2::new(10.0, 140.0)),
        ));
    }
}

impl Scene for MainMenu {
    fn update(&mut self, ctx: &mut Backend, cmd: &mut Sender<SceneCommand>) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Backend) -> GameResult {
        let res = Resources::default();

        let mut query = <(&Position, &Text)>::query();
        for (position, text) in query.iter_mut(&mut self.world) {
            ctx.draw_text(text, position)?;
        }

        Ok(())
    }
    fn on_input(&mut self, ctx: &mut Backend, button: &Button, state: &ButtonState, cmd: &mut Sender<SceneCommand>) -> GameResult {
        match button {
            Button::A => cmd.send(SceneCommand::GoTo(SceneRef("game_scene"))).unwrap(),
            Button::Start => cmd.send(SceneCommand::Exit).unwrap(),
            _ => (),
        }
        Ok(())
    }
}

