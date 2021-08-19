use bevy::{ecs::system::Command, prelude::*};
use std::collections::HashMap;

use super::mursten::*;

pub struct CurrentModel(pub Box<dyn Model>);

pub enum ModelChange {}

impl Model for CurrentModel {
    fn set_artifact(&mut self, aref: ArtifactReference, artifact: Artifact) {
        self.0.set_artifact(aref, artifact)
    }
    fn remove_artifact(&mut self, aref: &ArtifactReference) {
        self.0.remove_artifact(aref)
    }
    fn get_artifact(&self, aref: &ArtifactReference) -> Option<&Artifact> {
        self.0.get_artifact(aref)
    }
    fn list_artifacts(&self) -> Vec<ArtifactReference> {
        self.0.list_artifacts()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct BlockInstanceCreationError(String);

trait Instance {
    fn of_artifact<'a>(&'a self) -> &'a str;
}

struct CreateInstance {
    aref: ArtifactReference,
    root: Entity,
}

impl Command for CreateInstance {
    fn write(self: Box<Self>, world: &mut World) {
        let a = world
            .get_resource::<CurrentModel>()
            .unwrap()
            .get_artifact(&self.aref)
            .unwrap();
        world.build_artifact(&self.aref, self.root);
    }
}

struct BlockFactory {
    create_instance: Box<
        dyn Fn(Entity, &mut World) -> Result<HashMap<SlotName, Entity>, BlockInstanceCreationError>,
    >,
}

trait ArtifactInstantiation {
    fn get_factory(&self, aref: &ArtifactReference) -> BlockFactory;
    fn register_factory(&self, aref: ArtifactReference, factory: BlockFactory);

    fn alloc(&mut self) -> Entity;
    fn destroy(&mut self, root: Entity);
    fn free(&mut self, root: Entity);

    fn build_artifact(
        &mut self,
        aref: &ArtifactReference,
        root: Entity,
    ) -> HashMap<SlotName, Entity>;
    fn build_structure(&mut self, structure: &Structure, root: Entity)
        -> HashMap<SlotName, Entity>;
}

impl ArtifactInstantiation for World {
    fn get_factory(&self, aref: &ArtifactReference) -> BlockFactory {
        todo!()
    }
    fn register_factory(&self, aref: ArtifactReference, factory: BlockFactory) {
        todo!()
    }
    fn alloc(&mut self) -> Entity {
        todo!()
    }
    fn destroy(&mut self, root: Entity) {
        todo!()
    }
    fn free(&mut self, root: Entity) {
        todo!()
    }
    fn build_artifact(
        &mut self,
        aref: &ArtifactReference,
        root: Entity,
    ) -> HashMap<SlotName, Entity> {
        let artifact: Artifact = (*self
            .get_resource::<CurrentModel>()
            .unwrap()
            .get_artifact(&aref)
            .expect("Artifact not found in model"))
        .clone();

        match artifact {
            Artifact::Block(_block) => {
                let factory = self.get_factory(aref);
                (factory.create_instance)(root, self).expect("Block creation error")
            }
            Artifact::Structure(structure) => self.build_structure(&structure, root),
        }
    }

    fn build_structure(
        &mut self,
        structure: &Structure,
        root: Entity,
    ) -> HashMap<SlotName, Entity> {
        let main_slots = self.build_artifact(&structure.a_ref, root);

        let mut exposed_slots = HashMap::new();
        for (slot_name, entity) in main_slots {
            match structure.c.get(&slot_name) {
                Some(connection) => match connection {
                    Connection::Structure(structure) => {
                        self.build_structure(structure, root);
                    }
                    Connection::Slot(slot_name) => {
                        exposed_slots.insert(slot_name.clone(), entity);
                    }
                },
                None => panic!("Err(YCreateError::NoCforS(s))"),
            }
        }
        exposed_slots
    }
}

#[test]
#[ignore]
fn create_instance_of_basic_block() {
    todo!(
        "
            - Define a factory for a basic block with one component.
            - In an empty world instance one of those artifacts.
            - Check if the component got created.
            "
    )
}
