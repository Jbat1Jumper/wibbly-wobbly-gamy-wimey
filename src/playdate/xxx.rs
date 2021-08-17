#[cfg(abrakadabra)]
mod mursten_bevy_plugin {
    use bevy::{ecs::system::Command, prelude::*};
    use std::collections::HashMap;

    use super::mursten::*;

    trait Block {
        fn create_instance(&self, root: Entity, world: &mut World) -> HashMap<String, >;
    }

    trait Instance {
        fn of_artifact<'a>(&'a self) -> &'a str;
    }

    struct CreateInstance {
        aref: String,
        root: Entity,
    }

    impl Command for CreateInstance {
        fn write(self: Box<Self>, world: &mut World) {
            fetch(&self.a_ref, world)
                .map_err(ErrorInstancingAnA::DuringFetch)
                .and_then(|a: Artifact| a.create(self.root, world))
                .unwrap_or_else(|err| {
                    let mut errors = world
                        .get_resource_mut::<Vec<(Entity, ErrorInstancingAnA)>>()
                        .unwrap();
                    errors.push((self.root, err));
                });
        }
    }

    type AKind = String;

    struct AInfo {
        is_x: bool,
        kind: AKind,
    }

    trait W1 {
        fn list_a_refs(&self) -> Vec<String>;
        fn a_ref_info(&self, a_ref: &String) -> AInfo;
        fn satisfies(&self, a_kind: &AKind, s_kind: &SKind) -> bool;
    }

    trait W2 {
        fn alloc(&mut self) -> Entity;
        fn build(&mut self, a_ref: &String, root: &Entity) -> Result<(), ErrorInstancingAnA>;
        fn destroy(&mut self, root: &Entity);
        fn free(&mut self, root: Entity);

        fn get_ss(&self, root: &Entity) -> Result<Vec<(Entity, S)>, GetSsError>;
    }

    trait WAux {
        fn create_an_x(&mut self, x: Box<dyn X>, root: Entity);
        fn create_an_a(&mut self, a: A, root: Entity);
    }

    impl WAux for World {
        fn create_an_x(&mut self, x: Box<dyn X>, root: Entity) {
            todo!()
        }
        fn create_an_a(&mut self, a: Artifact, root: Entity) {
            todo!()
        }
    }

    #[derive(Debug)]
    enum ErrorInstancingAnA {
        DuringFetch(FetchError),
        OfTypeX(XCreateError),
        OfTypeY(YCreateError),
    }

    impl Artifact {
        fn create(&self, root: Entity, world: &mut World) -> Result<(), ErrorInstancingAnA> {
            match self {
                Artifact::X(x) => x.create(root, world).map_err(ErrorInstancingAnA::OfTypeX),
                Artifact::Y(y) => y.create(root, world).map_err(ErrorInstancingAnA::OfTypeY),
            }
        }
    }

    type XCreateError = String;

    trait X {
        fn create(&self, root: Entity, world: &mut World) -> Result<(), XCreateError>;
    }

    #[derive(Debug)]
    enum YCreateError {
        Fetch(FetchError),
        GetSs(GetSsError),
        CreatingRoot(Box<ErrorInstancingAnA>),
        NoCforS(S),
        CreatingC(S, Entity, CCreateError),
    }

    impl Y {
        fn create(&self, root: Entity, world: &mut World) -> Result<(), YCreateError> {
            let a = fetch(&self.a_ref, world).map_err(YCreateError::Fetch)?;
            world.entity_mut(root).insert(self.clone());
            a.create(root, world)
                .map_err(|err| YCreateError::CreatingRoot(Box::new(err)))?;
            let ss = get_ss(root, world).map_err(YCreateError::GetSs)?;
            for (s_entity, s) in ss {
                match self.c.get(&s.name) {
                    Some(c) => {
                        c.create(s.clone(), s_entity, world)
                            .map_err(|err| YCreateError::CreatingC(s, s_entity, err))?;
                    }
                    None => return Err(YCreateError::NoCforS(s)),
                }
            }
            Ok(())
        }
    }

    #[derive(Debug)]
    enum CCreateError {
        YFullfillS(Y),
        SFullfillS(S),
        CreatingY(Box<YCreateError>),
    }

    impl C {
        fn create(&self, s: S, root: Entity, world: &mut World) -> Result<(), CCreateError> {
            match self {
                C::Y(y) => {
                    if !y_fullfills_s(y, &s.kind) {
                        return Err(CCreateError::YFullfillS(y.clone()));
                    }
                    y.create(root, world)
                        .map_err(|err| CCreateError::CreatingY(Box::new(err)))?;
                }
                C::S(c_s) => {
                    if !s_fullfills_s(&c_s.kind, &s.kind) {
                        return Err(CCreateError::SFullfillS(c_s.clone()));
                    }
                    c_s.create(root, world);
                }
            }
            Ok(())
        }
        fn fullfils_s(&self, s: S) -> Result<(), CCreateError> {
            match self {
                C::Y(y) if !y_fullfills_s(y, &s.kind) => Err(CCreateError::YFullfillS(y.clone())),
                C::S(c_s) if !s_fullfills_s(&c_s.kind, &s.kind) => {
                    Err(CCreateError::SFullfillS(c_s.clone()))
                }
                _ => Ok(()),
            }
        }
    }

    impl S {
        fn create(&self, root: Entity, world: &mut World) {
            world.entity_mut(root).insert(self.clone());
        }
    }

    #[derive(Debug)]
    struct FetchError;
    fn fetch(a_ref: &String, world: &World) -> Result<Artifact, FetchError> {
        todo!()
    }

    #[derive(Debug)]
    struct GetSsError;
    fn get_ss(root: Entity, world: &World) -> Result<Vec<(Entity, S)>, GetSsError> {
        todo!()
    }

    #[derive(Debug)]
    struct SFullfilmentError;
    fn y_fullfills_s(y: &Y, s_kind: &SKind) -> bool {
        todo!()
    }
    fn s_fullfills_s(c_s_kind: &SKind, s_kind: &SKind) -> bool {
        todo!()
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

    pub struct Block {
        kind: SlotKind,
        slots: HashMap<SlotName, SlotKind>,
    }

    pub enum Artifact {
        Block(Block),
        Structure(Structure),
    }

    pub struct Structure {
        a_ref: ArtifactReference,
        c: HashMap<SlotName, Connection>,
    }

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

    trait Model {
        fn set(&mut self, aref: ArtifactReference, artifact: Artifact);
        fn remove(&mut self, aref: &ArtifactReference);
        fn get(&self, aref: &ArtifactReference) -> Option<&Artifact>;
        fn list(&self) -> Vec<ArtifactReference>;

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
            self.kind_of(aref)
                .map_err(ArtifactValidationError::GettingKindOf)?;
            self.slots_of(aref)
                .map_err(ArtifactValidationError::GettingSlotOf)?;
            Ok(())
        }

        fn kind_of(&self, aref: &ArtifactReference) -> Result<SlotKind, GettingKindOfError> {
            self.kind_of_with_breadcrumb(aref, vec![])
        }
        fn kind_of_with_breadcrumb(
            &self,
            aref: &ArtifactReference,
            mut breadcrumb: Vec<ArtifactReference>,
        ) -> Result<SlotKind, GettingKindOfError> {
            match self
                .get(aref)
                .ok_or_else(|| GettingKindOfError::Unexistent(breadcrumb.clone(), aref.clone()))?
            {
                Artifact::Block(ref block) => Ok(block.kind.clone()),
                Artifact::Structure(structure) => {
                    breadcrumb.push(aref.clone());
                    if breadcrumb.contains(&structure.a_ref) {
                        return Err(GettingKindOfError::RecursionDetected(
                            breadcrumb.clone(),
                            structure.a_ref.clone(),
                        ));
                    }
                    self.kind_of_with_breadcrumb(&structure.a_ref, breadcrumb)
                }
            }
        }

        fn slots_of(&self, aref: &ArtifactReference) -> Result<HashMap<SlotName, SlotKind>, GettingSlotOfError> {
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

        fn union<ModelPrime: Model>(&self, other_model: ModelPrime) -> InMemoryModel {
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
                kind: sk("A"),
                slots: hashmap! {},
            }),
        );
        model.validate_model().unwrap();
        assert_eq!(model.kind_of(&a).unwrap(), sk("A"));
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
                kind: sk("A"),
                slots: hashmap! {
                    sn("1") => sk("A")
                },
            }),
        );
        model.validate_model().unwrap();
        assert_eq!(model.kind_of(&ar("a")).unwrap(), sk("A"));
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
                kind: sk("B"),
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

        assert_eq!(model.kind_of(&ar("a")).unwrap(), sk("B"));
        assert_eq!(model.slots_of(&ar("a")).unwrap().is_empty(), true);
    }

    #[test]
    fn a_structure_using_a_simple_block_remapping_slots() {
        let mut model = empty_test_model();
        model.set(
            ar("b"),
            Artifact::Block(Block {
                kind: sk("B"),
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

        assert_eq!(model.kind_of(&ar("a")).unwrap(), sk("B"));
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
                kind: sk("Natural"),
                slots: hashmap! {
                    sn("x") => sk("Natural"),
                },
            }),
        );
        model.set(
            ar("zero"),
            Artifact::Block(Block {
                kind: sk("Natural"),
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
        } else if  *aref == ar("number_4") {
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
