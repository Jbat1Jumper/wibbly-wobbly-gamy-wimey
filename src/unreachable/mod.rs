use crate::common::*;
use crate::fw::Plugin;
use legion::{Resources, Schedule, World};

mod scenes;

pub struct UnreachableGame;

impl Plugin for UnreachableGame {
    fn name(&self) -> String {
        "UnreachableGame".into()
    }

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

    fn load_scene(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
        scene: SceneRef,
    ) -> Option<Schedule> {
        scenes::load_scene(world, resources, scene)
    }
}
