/*
use bevy::prelude::*;
use bevy::utils::HashMap;
use super::skeleton;

pub struct Root {
    pub definition: Entity,
    pub parts: HashMap<skeleton::LinkRef, Entity>,
    pose: skeleton::Pose,
}

pub struct CreateInstance {
    definition: Entity,
    target: Entity,
    pose: skeleton::Pose,
}

impl bevy::ecs::system::Command for CreateInstance {
    fn write(self: Box<Self>, world: &mut World) {
        let definition = world
            .entity(self.definition)
            .get::<skeleton::Definition>()
            .expect("Definition not found when creating instance")
            .clone();

        let mut parts = HashMap::default();
        for (LinkRef, _) in definition.entries.iter() {
            parts.insert(*part_id, world.spawn().id());
        }
        for (part_id, part_entry) in definition.entries.iter() {
            match part_entry.part {
                skeleton::Part::Translation(x, y, z) => {}
                skeleton::Part::Rotation() => {}
            }
        }

        let root = world.entity_mut(self.target);
    }
}

pub struct UpdatePose {
    target: Entity,
    new_pose: skeleton::Pose,
}
*/
