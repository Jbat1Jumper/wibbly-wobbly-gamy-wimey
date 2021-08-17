mod mursten_bevy_plugin {
    use bevy::{ecs::system::Command, prelude::*};
    use std::collections::HashMap;

    use super::mursten::*;

    // TODO: This could probably be in another module separated from bevy.
    // Some things like a trait for a ModelRepository with crud operations and model resolution
    // algorithms can be used without the bevy application. For instance, implementing a model repository
    // on top of a distributed network is something agnostic to bevy but directly usable by this
    // library.
    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct ModelReference(String);
    pub fn mr<T: Into<String>>(x: T) -> ModelReference {
        ModelReference(x.into())
    }

    enum ModelChange {}

    pub trait ModelRepository {
        fn list_models(&mut self) -> Vec<ModelReference>;
        fn delete_model(&self, mref: &ModelReference);
        fn get_model(&self, mref: &ModelReference) -> &dyn Model;
        fn create_empty_model(&mut self, mref: ModelReference);
        fn change_model(&mut self, mref: &ModelReference, change: ModelChange);
    }

    impl ModelRepository for World {
        fn list_models(&mut self) -> Vec<ModelReference> {
            todo!()
        }
        fn delete_model(&self, mref: &ModelReference) {
            todo!()
        }
        fn get_model(&self, mref: &ModelReference) -> &dyn Model {
            todo!()
        }
        fn create_empty_model(&mut self, mref: ModelReference) {
            todo!()
        }
        fn change_model(&mut self, mref: &ModelReference, change: ModelChange) {
            todo!()
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct BlockInstanceCreationError(String);

    trait Instance {
        fn of_artifact<'a>(&'a self) -> &'a str;
    }

    struct CreateInstance {
        aref: ArtifactReference,
        mref: ModelReference,
        root: Entity,
    }

    impl Command for CreateInstance {
        fn write(self: Box<Self>, world: &mut World) {
            let a = world.get_model(&self.mref).get(&self.aref).unwrap();
            world.build_artifact(&self.mref, &self.aref, self.root);
        }
    }

    struct BlockFactory {
        create_instance: Box<
            dyn Fn(
                Entity,
                &mut World,
            ) -> Result<HashMap<SlotName, Entity>, BlockInstanceCreationError>,
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
            mref: &ModelReference,
            aref: &ArtifactReference,
            root: Entity,
        ) -> HashMap<SlotName, Entity>;
        fn build_structure(
            &mut self,
            mref: &ModelReference,
            structure: &Structure,
            root: Entity,
        ) -> HashMap<SlotName, Entity>;
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
            mref: &ModelReference,
            aref: &ArtifactReference,
            root: Entity,
        ) -> HashMap<SlotName, Entity> {
            let artifact: Artifact = (*self
                .get_model(&mref)
                .get(&aref)
                .expect("Artifact not found in model"))
            .clone();

            match artifact {
                Artifact::Block(_block) => {
                    let factory = self.get_factory(aref);
                    (factory.create_instance)(root, self).expect("Block creation error")
                }
                Artifact::Structure(structure) => self.build_structure(mref, &structure, root),
            }
        }

        fn build_structure(
            &mut self,
            mref: &ModelReference,
            structure: &Structure,
            root: Entity,
        ) -> HashMap<SlotName, Entity> {
            let main_slots = self.build_artifact(mref, &structure.a_ref, root);

            let mut exposed_slots = HashMap::new();
            for (slot_name, entity) in main_slots {
                match structure.c.get(&slot_name) {
                    Some(connection) => match connection {
                        Connection::Structure(structure) => {
                            self.build_structure(mref, structure, root);
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
}

mod mursten {
    #[macro_use]
    pub use maplit::hashmap;
    use std::collections::HashMap;

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct ArtifactReference(String);
    pub fn ar<T: Into<String>>(x: T) -> ArtifactReference {
        ArtifactReference(x.into())
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct SlotName(String);
    pub fn sn<T: Into<String>>(x: T) -> SlotName {
        SlotName(x.into())
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct SlotKind(String);
    pub fn sk<T: Into<String>>(x: T) -> SlotKind {
        SlotKind(x.into())
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Block {
        pub main_slot_kind: SlotKind,
        pub slots: HashMap<SlotName, SlotKind>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum Artifact {
        Block(Block),
        Structure(Structure),
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Structure {
        pub a_ref: ArtifactReference,
        pub c: HashMap<SlotName, Connection>,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum Connection {
        Structure(Structure),
        Slot(SlotName),
    }

    #[derive(Debug)]
    pub enum ModelValidationError {
        Artifact(ArtifactReference, ArtifactValidationError),
    }

    #[derive(Debug)]
    pub enum ArtifactValidationError {
        GettingKindOf(GettingKindOfError),
        GettingSlotOf(GettingSlotOfError),
    }

    #[derive(Debug)]
    pub enum GettingKindOfError {
        Unexistent(Vec<ArtifactReference>, ArtifactReference),
        RecursionDetected(Vec<ArtifactReference>, ArtifactReference),
    }
    #[derive(Debug)]
    pub struct GettingSlotOfError;

    pub trait Model {
        fn set(&mut self, aref: ArtifactReference, artifact: Artifact);
        fn remove(&mut self, aref: &ArtifactReference);
        fn get(&self, aref: &ArtifactReference) -> Option<&Artifact>;

        // TODO: This could be an iterator to easily support infinite models...
        fn list(&self) -> Vec<ArtifactReference>;
        // ... and expose something like this to hint users of what to expect of this.
        fn is_finite(&self) -> bool {
            true
        }

        fn is_all_valid(&self) -> bool {
            self.validate_model().is_ok()
        }

        fn validate_model(&self) -> Result<(), ModelValidationError> {
            for aref in self.list().into_iter() {
                self.validate(&aref)
                    .map_err(|err| ModelValidationError::Artifact(aref, err))?;
            }
            Ok(())
        }

        fn validate(&self, aref: &ArtifactReference) -> Result<(), ArtifactValidationError> {
            self.main_slot_kind_of(aref)
                .map_err(ArtifactValidationError::GettingKindOf)?;
            self.slots_of(aref)
                .map_err(ArtifactValidationError::GettingSlotOf)?;
            Ok(())
        }

        fn main_slot_kind_of(
            &self,
            aref: &ArtifactReference,
        ) -> Result<SlotKind, GettingKindOfError> {
            self.main_slot_kind_of_with_breadcrumb(aref, vec![])
        }
        fn main_slot_kind_of_with_breadcrumb(
            &self,
            aref: &ArtifactReference,
            mut breadcrumb: Vec<ArtifactReference>,
        ) -> Result<SlotKind, GettingKindOfError> {
            match self
                .get(aref)
                .ok_or_else(|| GettingKindOfError::Unexistent(breadcrumb.clone(), aref.clone()))?
            {
                Artifact::Block(ref block) => Ok(block.main_slot_kind.clone()),
                Artifact::Structure(structure) => {
                    breadcrumb.push(aref.clone());
                    if breadcrumb.contains(&structure.a_ref) {
                        return Err(GettingKindOfError::RecursionDetected(
                            breadcrumb.clone(),
                            structure.a_ref.clone(),
                        ));
                    }
                    self.main_slot_kind_of_with_breadcrumb(&structure.a_ref, breadcrumb)
                }
            }
        }

        fn slots_of(
            &self,
            aref: &ArtifactReference,
        ) -> Result<HashMap<SlotName, SlotKind>, GettingSlotOfError> {
            match self.get(aref).unwrap() {
                Artifact::Block(ref block) => Ok(block.slots.clone()),
                Artifact::Structure(structure) => {
                    if *aref == structure.a_ref {
                        return Err(GettingSlotOfError);
                    }
                    self.slots_of_structure(structure)
                }
            }
        }
        fn slots_of_structure(
            &self,
            structure: &Structure,
        ) -> Result<HashMap<SlotName, SlotKind>, GettingSlotOfError> {
            let inner_slots = self.slots_of(&structure.a_ref)?;

            let mut slots = hashmap! {};

            for (slot_name, slot_kind) in inner_slots {
                match structure.c.get(&slot_name) {
                    Some(c) => match c {
                        Connection::Slot(outer_slot_name) => {
                            slots.insert(outer_slot_name.clone(), slot_kind.clone());
                        }
                        Connection::Structure(child_structure) => {
                            for (slot_name, slot_kind) in
                                self.slots_of_structure(child_structure)?
                            {
                                slots.insert(slot_name, slot_kind);
                            }
                        }
                    },
                    None => return Err(GettingSlotOfError),
                }
            }

            Ok(slots)
        }

        fn dependencies(&self, aref: &ArtifactReference) -> Vec<ArtifactReference> {
            todo!()
        }

        fn dependents(&self, aref: &ArtifactReference) -> Vec<ArtifactReference> {
            todo!()
        }

        fn union(&self, other_model: &dyn Model) -> InMemoryModel {
            todo!()
        }
    }

    pub struct Location(Vec<SlotName>);

    impl Artifact {
        fn composite(&self) -> bool {
            if let Artifact::Structure(_) = self {
                true
            } else {
                false
            }
        }
    }

    impl Structure {
        fn swap(&mut self, target: Location, new_aref: ArtifactReference) {
            todo!();
        }

        fn replace(&mut self, target: Location, new_structure: Structure) {
            todo!();
        }

        fn connect(&mut self, target: Location, slot_name: SlotName, connection: Connection) {
            todo!();
        }
    }

    pub struct InMemoryModel(HashMap<ArtifactReference, Artifact>);

    impl Model for InMemoryModel {
        fn set(&mut self, aref: ArtifactReference, artifact: Artifact) {
            self.0.insert(aref, artifact);
        }

        fn remove(&mut self, aref: &ArtifactReference) {
            self.0.remove(aref);
        }

        fn get(&self, aref: &ArtifactReference) -> Option<&Artifact> {
            self.0.get(aref)
        }
        fn list(&self) -> Vec<ArtifactReference> {
            self.0.keys().cloned().collect()
        }
    }

    fn empty_test_model() -> InMemoryModel {
        InMemoryModel(hashmap! {})
    }

    #[test]
    fn empty_model_is_valid() {
        let mut model = empty_test_model();
        model.validate_model().unwrap();
        assert!(model.list().is_empty());
    }

    #[test]
    fn a_simple_block() {
        let mut model = empty_test_model();
        let a = ar("a");
        model.set(
            a.clone(),
            Artifact::Block(Block {
                main_slot_kind: sk("A"),
                slots: hashmap! {},
            }),
        );
        model.validate_model().unwrap();
        assert_eq!(model.main_slot_kind_of(&a).unwrap(), sk("A"));
        assert_eq!(model.slots_of(&a).unwrap(), hashmap! {});

        let artifact = model.get(&a).unwrap();
        assert_eq!(artifact.composite(), false);

        assert!(model.list().contains(&a));
    }

    #[test]
    fn a_block_with_a_slot() {
        let mut model = empty_test_model();
        model.set(
            ar("a"),
            Artifact::Block(Block {
                main_slot_kind: sk("A"),
                slots: hashmap! {
                    sn("1") => sk("A")
                },
            }),
        );
        model.validate_model().unwrap();
        assert_eq!(model.main_slot_kind_of(&ar("a")).unwrap(), sk("A"));
        assert_eq!(model.slots_of(&ar("a")).unwrap()[&sn("1")], sk("A"));

        let artifact = model.get(&ar("a")).unwrap();
        assert_eq!(artifact.composite(), false);

        assert!(model.list().contains(&ar("a")));
    }

    #[test]
    fn a_structure_depending_on_an_unexistent_artifact() {
        let mut model = empty_test_model();
        model.set(
            ar("a"),
            Artifact::Structure(Structure {
                a_ref: ar("b"),
                c: hashmap! {},
            }),
        );
        model.validate_model().expect_err("Should fail");

        assert!(model.list().contains(&ar("a")));
        assert!(model.get(&ar("a")).unwrap().composite());
    }

    #[test]
    fn a_structure_using_a_simple_block() {
        let mut model = empty_test_model();
        model.set(
            ar("b"),
            Artifact::Block(Block {
                main_slot_kind: sk("B"),
                slots: hashmap! {},
            }),
        );
        model.set(
            ar("a"),
            Artifact::Structure(Structure {
                a_ref: ar("b"),
                c: hashmap! {},
            }),
        );
        model.validate_model().unwrap();

        assert_eq!(model.main_slot_kind_of(&ar("a")).unwrap(), sk("B"));
        assert_eq!(model.slots_of(&ar("a")).unwrap().is_empty(), true);
    }

    #[test]
    fn a_structure_using_a_simple_block_remapping_slots() {
        let mut model = empty_test_model();
        model.set(
            ar("b"),
            Artifact::Block(Block {
                main_slot_kind: sk("B"),
                slots: hashmap! {
                    sn("80") => sk("PROTO")
                },
            }),
        );
        model.set(
            ar("a"),
            Artifact::Structure(Structure {
                a_ref: ar("b"),
                c: hashmap! {
                    sn("80") => Connection::Slot(sn("9980")),
                },
            }),
        );
        model.validate_model().unwrap();

        assert_eq!(model.main_slot_kind_of(&ar("a")).unwrap(), sk("B"));
        assert_eq!(
            model.slots_of(&ar("a")).unwrap(),
            hashmap! {sn("9980") => sk("PROTO")}
        );
    }

    #[test]
    fn a_structure_depending_on_itself() {
        let mut model = empty_test_model();
        model.set(
            ar("a"),
            Artifact::Structure(Structure {
                a_ref: ar("a"),
                c: hashmap! {},
            }),
        );
        model.validate_model().expect_err("Should fail");
    }

    #[test]
    fn mutually_recursive_structures() {
        let mut model = empty_test_model();
        model.set(
            ar("a"),
            Artifact::Structure(Structure {
                a_ref: ar("b"),
                c: hashmap! {},
            }),
        );
        model.set(
            ar("b"),
            Artifact::Structure(Structure {
                a_ref: ar("a"),
                c: hashmap! {},
            }),
        );
        model.validate_model().expect_err("Should fail");
    }

    fn peano_model() -> InMemoryModel {
        let mut model = empty_test_model();
        model.set(
            ar("successor"),
            Artifact::Block(Block {
                main_slot_kind: sk("Natural"),
                slots: hashmap! {
                    sn("x") => sk("Natural"),
                },
            }),
        );
        model.set(
            ar("zero"),
            Artifact::Block(Block {
                main_slot_kind: sk("Natural"),
                slots: hashmap! {},
            }),
        );
        model
    }

    #[test]
    fn peano_numbers() {
        let mut model = peano_model();
        model.set(
            ar("number_2"),
            Artifact::Structure(Structure {
                a_ref: ar("successor"),
                c: hashmap! {
                    sn("x") => Connection::Structure(Structure {
                        a_ref: ar("successor"),
                        c: hashmap!{
                            sn("x") => Connection::Structure(Structure {
                                a_ref: ar("zero"),
                                c: hashmap!{}
                            })
                        }
                    })
                },
            }),
        );
        model.validate_model().unwrap();
        assert!(model.slots_of(&ar("number_2")).unwrap().is_empty());
        assert_eq!(peano_eval(&ar("number_2"), &model), 2);
    }

    fn peano_eval<M: Model>(aref: &ArtifactReference, model: &M) -> usize {
        if *aref == ar("number_2") {
            2
        } else if *aref == ar("number_4") {
            4
        } else {
            todo!("Not implemented")
        }
    }

    #[test]
    fn two_and_two_is_four() {
        let mut model = peano_model();
        model.set(
            ar("plus_2"),
            Artifact::Structure(Structure {
                a_ref: ar("successor"),
                c: hashmap! {
                    sn("x") => Connection::Structure(Structure {
                        a_ref: ar("successor"),
                        c: hashmap!{
                            sn("x") => Connection::Slot(sn("x"))
                        }
                    })
                },
            }),
        );
        model.set(
            ar("number_4"),
            Artifact::Structure(Structure {
                a_ref: ar("plus_2"),
                c: hashmap! {
                    sn("x") => Connection::Structure(Structure {
                        a_ref: ar("plus_2"),
                        c: hashmap!{
                            sn("x") => Connection::Structure(Structure {
                                a_ref: ar("zero"),
                                c: hashmap!{}
                            })
                        }
                    })
                },
            }),
        );
        model.validate_model().unwrap();
        assert_eq!(model.slots_of(&ar("plus_2")).unwrap().keys().count(), 1);
        assert_eq!(
            model.slots_of(&ar("plus_2")).unwrap()[&sn("x")],
            sk("Natural")
        );
        assert!(model.slots_of(&ar("number_4")).unwrap().is_empty());

        assert_eq!(peano_eval(&ar("number_4"), &model), 4);
    }

    #[test]
    #[ignore]
    fn xxx() {
        todo!("Assert that structure expansion equals to an equivalent structure depending only on blocks");
    }
}
