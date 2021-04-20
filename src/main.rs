use std::env;
use std::path;

#[macro_use]
mod common;
mod fw;
mod ggez_backend;
mod unreachable;
mod physics;

#[macro_use]
use common::*;
use bevy::prelude::*;

use physics::PhysicsPlugin;
use pyxel_plugin::PyxelPlugin;
use unreachable::UnreachableGame;

pub fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(PyxelPlugin)
        .add_plugin(UnreachableGame)
        .add_plugin(PhysicsPlugin)
        .add_startup_system(on_startup.system())
        .run();
}

fn on_startup() {
    println!("Started!");
}
