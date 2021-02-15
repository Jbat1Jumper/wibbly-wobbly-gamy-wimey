pub mod game;
pub mod intro;
pub mod main_menu;

use crate::common::SceneRef;
use legion::{Resources, World, Schedule};

pub fn load_scene(world: &mut World, resources: &mut Resources, scene: SceneRef) -> Option<Schedule> {
    let SceneRef(scene) = scene;
    Some(match scene {
        "intro" => intro::Intro::init(world, resources),
        "main_menu" => main_menu::MainMenu::init(world, resources),
        "game" => game::GameScene::init(world, resources),
        _ => return None,
    })
}
