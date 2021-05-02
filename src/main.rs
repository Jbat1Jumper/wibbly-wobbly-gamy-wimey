#[macro_use]
mod common;
mod pyxel_plugin;
mod unreachable;

#[macro_use]
use bevy::prelude::*;
use heron::prelude::*;

use pyxel_plugin::PyxelPlugin;
use unreachable::UnreachableGame;

pub fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(PyxelPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(UnreachableGame)
        .add_startup_system(on_startup.system())
        .add_startup_system(common::known_fonts::load_known_fonts.system())
        .run();
}

fn on_startup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    println!("Started!");
}
