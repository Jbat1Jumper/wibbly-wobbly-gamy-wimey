use crate::common::*;
use bevy::prelude::*;

mod scenes;

pub struct UnreachableGame;

impl Plugin for UnreachableGame {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(scenes::LoadGameScenes);
    }
}
