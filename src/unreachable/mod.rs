use crate::fw::Plugin;
use crate::common::*;
use legion::{World, Resources, Schedule};

mod scenes;

pub struct UnreachableGame;

impl Plugin for UnreachableGame {
    fn init(&mut self, world: &mut World, resources: &mut Resources) {
        resources.insert(PyxelFiles(map! {
            "base.pyxel" => {
                //pyxel::open("resources/base.pyxel")
                pyxel::load_from_memory(
                    include_bytes!("resources/base.pyxel")
                )
                .expect("Problems loading base.pyxel")
            }
        }));
    }

    fn load_scene(&mut self, world: &mut World, resources: &mut Resources, scene: SceneRef) -> Option<Schedule> {
        scenes::load_scene(world, resources, scene)
    }
}
