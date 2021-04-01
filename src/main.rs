use ggez;
use glam::f32::Vec2;
use legion::*;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

#[macro_use]
mod common;
mod fw;
mod ggez_backend;
mod unreachable;
mod physics;

#[macro_use]
use common::*;

use ggez_backend::{GgezBackend, SpriteResources};
use unreachable::UnreachableGame;
use physics::PhysicsPlugin;

pub fn main() {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("Unreachable", "ggez").add_resource_path(resource_dir);

    fw::Game::build()
        .using(GgezBackend::new(cb.build().unwrap(), SpriteResources::default()).unwrap())
        .using(UnreachableGame)
        .using(PhysicsPlugin)
        .run(SceneRef("intro"));
}
