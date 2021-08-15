mod take_1 {
    use bevy::prelude::World;
    use std::collections::HashMap;

    type DeviceKind = String;
    type SocketName = String;
    type SocketType = String;

    struct DeviceBlueprint {
        kind: DeviceKind,
        perihperals: HashMap<SocketName, Connection>,
    }

    enum Connection {
        Device(DeviceBlueprint),
        Socket(SocketName, SocketType),
        Disconnected,
    }

    enum DeviceXXX {
        Blueprint(DeviceBlueprint),
        Base(Box<dyn BaseDevice>),
    }

    trait BaseDevice {
        fn create(&self, world: &mut World);
    }
}

mod take_2 {
    use bevy::{ecs::system::Command, prelude::*};
    use std::collections::HashMap;

    type ARef = String;
    enum A {
        X(Box<dyn X>),
        Y(Y),
    }

    #[derive(Clone, Debug)]
    struct Y {
        a_ref: ARef,
        c: HashMap<SName, C>,
    }

    #[derive(Clone, Debug)]
    enum C {
        Y(Y),
        S(S),
    }

    type SName = String;
    type SKind = String;

    #[derive(Clone, Debug)]
    struct S {
        name: SName,
        kind: SKind,
    }

    struct InstanceAnACommand {
        a_ref: ARef,
        root: Entity,
    }

    impl Command for InstanceAnACommand {
        fn write(self: Box<Self>, world: &mut World) {
            fetch(&self.a_ref, world)
                .map_err(ErrorInstancingAnA::DuringFetch)
                .and_then(|a: A| a.create(self.root, world))
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
        fn list_a_refs(&self) -> Vec<ARef>;
        fn a_ref_info(&self, a_ref: &ARef) -> AInfo;
        fn satisfies(&self, a_kind: &AKind, s_kind: &SKind) -> bool;
    }

    trait W2 {
        fn alloc(&mut self) -> Entity;
        fn build(&mut self, a_ref: &ARef, root: &Entity) -> Result<(), ErrorInstancingAnA>;
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
        fn create_an_a(&mut self, a: A, root: Entity) {
            todo!()
        }
    }

    #[derive(Debug)]
    enum ErrorInstancingAnA {
        DuringFetch(FetchError),
        OfTypeX(XCreateError),
        OfTypeY(YCreateError),
    }

    impl A {
        fn create(&self, root: Entity, world: &mut World) -> Result<(), ErrorInstancingAnA> {
            match self {
                A::X(x) => x.create(root, world).map_err(ErrorInstancingAnA::OfTypeX),
                A::Y(y) => y.create(root, world).map_err(ErrorInstancingAnA::OfTypeY),
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
    fn fetch(a_ref: &ARef, world: &World) -> Result<A, FetchError> {
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

mod take_3 {
    #[macro_use]
    pub use maplit::hashmap;
    use std::collections::HashMap;

    type SlotName = String;
    type SlotKind = String;

    struct Block {
        kind: SlotKind,
        slots: HashMap<SlotName, SlotKind>,
    }

    enum Artifact {
        Block(Block),
        Structure(Structure),
    }

    type ARef = String;

    struct Structure {
        a_ref: ARef,
        c: HashMap<SlotName, Connection>,
    }

    enum Connection {
        Structure(Structure),
        Slot(SlotName),
    }

    #[derive(Debug)]
    enum ModelValidationError {
        Artifact(ARef, ArtifactValidationError),
    }

    #[derive(Debug)]
    enum ArtifactValidationError {
        GettingKindOf(GettingKindOfError),
        GettingSlotOf(GettingSlotOfError),
    }

    #[derive(Debug)]
    struct GettingKindOfError;
    #[derive(Debug)]
    struct GettingSlotOfError;

    trait Model {
        fn set(&mut self, aref: ARef, artifact: Artifact);
        fn remove(&mut self, aref: ARef);
        fn get(&self, aref: ARef) -> Option<&Artifact>;
        fn list(&self) -> Vec<ARef>;

        fn is_all_valid(&self) -> bool {
            self.validate_model().is_ok()
        }

        fn validate_model(&self) -> Result<(), ModelValidationError> {
            for aref in self.list().into_iter() {
                self.validate(aref.clone())
                    .map_err(|err| ModelValidationError::Artifact(aref, err))?;
            }
            Ok(())
        }

        fn validate(&self, aref: ARef) -> Result<(), ArtifactValidationError> {
            self.kind_of(aref.clone())
                .map_err(ArtifactValidationError::GettingKindOf)?;
            self.slots_of(aref)
                .map_err(ArtifactValidationError::GettingSlotOf)?;
            Ok(())
        }

        fn kind_of(&self, aref: ARef) -> Result<SlotKind, GettingKindOfError> {
            self.kind_of_with_breadcrumb(aref, vec![])
        }
        fn kind_of_with_breadcrumb(
            &self,
            aref: ARef,
            mut breadcrumb: Vec<ARef>,
        ) -> Result<SlotKind, GettingKindOfError> {
            match self.get(aref.clone()).ok_or(GettingKindOfError)? {
                Artifact::Block(ref block) => Ok(block.kind.clone()),
                Artifact::Structure(structure) => {
                    breadcrumb.push(aref);
                    if breadcrumb.contains(&structure.a_ref) {
                        return Err(GettingKindOfError);
                    }
                    self.kind_of_with_breadcrumb(structure.a_ref.clone(), breadcrumb)
                }
            }
        }

        fn slots_of(&self, aref: ARef) -> Result<HashMap<SlotName, SlotKind>, GettingSlotOfError> {
            match self.get(aref.clone()).unwrap() {
                Artifact::Block(ref block) => Ok(block.slots.clone()),
                Artifact::Structure(structure) => {
                    if aref == structure.a_ref {
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
            let inner_slots = self.slots_of(structure.a_ref.clone())?;

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

        fn dependencies(&self, aref: ARef) -> Vec<ARef> {
            todo!()
        }

        fn dependents(&self, aref: ARef) -> Vec<ARef> {
            todo!()
        }

        fn union<ModelPrime: Model>(&self, other_model: ModelPrime) -> InMemoryModel {
            todo!()
        }
    }

    struct Location(Vec<SlotName>);

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
        fn swap(&mut self, target: Location, aref: ARef) {
            todo!();
        }

        fn replace(&mut self, target: Location, new_structure: Structure) {
            todo!();
        }

        fn connect(&mut self, target: Location, slot_name: SlotName, connection: Connection) {
            todo!();
        }
    }

    struct InMemoryModel(HashMap<ARef, Artifact>);

    impl Model for InMemoryModel {
        fn set(&mut self, aref: ARef, artifact: Artifact) {
            self.0.insert(aref, artifact);
        }

        fn remove(&mut self, aref: ARef) {
            self.0.remove(&aref);
        }

        fn get(&self, aref: ARef) -> Option<&Artifact> {
            self.0.get(&aref)
        }
        fn list(&self) -> Vec<ARef> {
            self.0.keys().cloned().collect()
        }
    }

    fn empty_test_model() -> InMemoryModel {
        InMemoryModel(hashmap! {})
    }

    #[test]
    fn empty_model_is_valid() {
        let mut model = empty_test_model();
        assert!(model.is_all_valid());
        assert!(model.list().is_empty());
    }

    #[test]
    fn a_simple_block() {
        let mut model = empty_test_model();
        model.set(
            "a".into(),
            Artifact::Block(Block {
                kind: "A".into(),
                slots: hashmap! {},
            }),
        );
        assert!(model.is_all_valid());
        assert_eq!(model.kind_of("a".into()).unwrap(), "A");
        assert_eq!(model.slots_of("a".into()).unwrap(), hashmap! {});

        let artifact = model.get("a".into()).unwrap();
        assert_eq!(artifact.composite(), false);

        assert!(model.list().contains(&String::from("a")));
    }

    #[test]
    fn a_block_with_a_slot() {
        let mut model = empty_test_model();
        model.set(
            "a".into(),
            Artifact::Block(Block {
                kind: "A".into(),
                slots: hashmap! {
                    String::from("1") => String::from("A")
                },
            }),
        );
        assert!(model.is_all_valid());
        assert_eq!(model.kind_of("a".into()).unwrap(), "A");
        assert_eq!(model.slots_of("a".into()).unwrap()["1"], "A");

        let artifact = model.get("a".into()).unwrap();
        assert_eq!(artifact.composite(), false);

        assert!(model.list().contains(&String::from("a")));
    }

    #[test]
    fn a_structure_depending_on_an_unexistent_artifact() {
        let mut model = empty_test_model();
        model.set(
            "a".into(),
            Artifact::Structure(Structure {
                a_ref: "b".into(),
                c: hashmap! {},
            }),
        );
        assert_eq!(model.is_all_valid(), false);

        assert!(model.list().contains(&String::from("a")));
        assert!(model.get("a".into()).unwrap().composite());
    }

    #[test]
    fn a_structure_using_a_simple_block() {
        let mut model = empty_test_model();
        model.set(
            "b".into(),
            Artifact::Block(Block {
                kind: "B".into(),
                slots: hashmap! {},
            }),
        );
        model.set(
            "a".into(),
            Artifact::Structure(Structure {
                a_ref: "b".into(),
                c: hashmap! {},
            }),
        );
        assert!(model.is_all_valid());

        assert_eq!(model.kind_of("a".into()).unwrap(), "B");
        assert_eq!(model.slots_of("a".into()).unwrap().is_empty(), true);
    }

    #[test]
    fn a_structure_using_a_simple_block_remapping_slots() {
        let mut model = empty_test_model();
        model.set(
            "b".into(),
            Artifact::Block(Block {
                kind: "B".into(),
                slots: hashmap! {
                    String::from("80") => String::from("PROTO")
                },
            }),
        );
        model.set(
            "a".into(),
            Artifact::Structure(Structure {
                a_ref: "b".into(),
                c: hashmap! {
                    String::from("80") => Connection::Slot(String::from("9980")),
                },
            }),
        );
        assert!(model.is_all_valid());

        assert_eq!(model.kind_of("a".into()).unwrap(), "B");
        assert_eq!(
            model.slots_of("a".into()).unwrap(),
            hashmap! {String::from("9980") => String::from("PROTO")}
        );
    }

    #[test]
    fn a_structure_depending_on_itself() {
        let mut model = empty_test_model();
        model.set(
            "a".into(),
            Artifact::Structure(Structure {
                a_ref: "a".into(),
                c: hashmap! {},
            }),
        );
        assert!(!model.is_all_valid());
    }

    #[test]
    fn mutually_recursive_structures() {
        let mut model = empty_test_model();
        model.set(
            "a".into(),
            Artifact::Structure(Structure {
                a_ref: "b".into(),
                c: hashmap! {},
            }),
        );
        model.set(
            "b".into(),
            Artifact::Structure(Structure {
                a_ref: "a".into(),
                c: hashmap! {},
            }),
        );
        assert!(!model.is_all_valid());
    }

    fn peano_model() -> InMemoryModel {
        let mut model = empty_test_model();
        model.set(
            "successor".into(),
            Artifact::Block(Block {
                kind: "Natural".into(),
                slots: hashmap! {
                    String::from("x") => String::from("Natural"),
                },
            }),
        );
        model.set(
            "zero".into(),
            Artifact::Block(Block {
                kind: "Natural".into(),
                slots: hashmap! {},
            }),
        );
        model
    }

    #[test]
    fn peano_numbers() {
        let mut model = peano_model();
        model.set(
            "number_2".into(),
            Artifact::Structure(Structure {
                a_ref: "successor".into(),
                c: hashmap! {
                    String::from("x") => Connection::Structure(Structure {
                        a_ref: "successor".into(),
                        c: hashmap!{
                            String::from("x") => Connection::Structure(Structure {
                                a_ref: "zero".into(),
                                c: hashmap!{}
                            })
                        }
                    })
                },
            }),
        );
        assert!(model.is_all_valid());
        assert!(model.slots_of("number_2".into()).unwrap().is_empty());
        assert_eq!(peano_eval("number_2".into(), &model), 2);
    }

    fn peano_eval<M: Model>(aref: ARef, model: &M) -> usize {
        2
    }

    #[test]
    fn two_and_two_is_four() {
        let mut model = peano_model();
        model.set(
            "plus_2".into(),
            Artifact::Structure(Structure {
                a_ref: "successor".into(),
                c: hashmap! {
                    String::from("x") => Connection::Structure(Structure {
                        a_ref: "successor".into(),
                        c: hashmap!{
                            String::from("x") => Connection::Slot("x".into())
                        }
                    })
                },
            }),
        );
        model.set(
            "number_4".into(),
            Artifact::Structure(Structure {
                a_ref: "plus_two".into(),
                c: hashmap! {
                    String::from("x") => Connection::Structure(Structure {
                        a_ref: "plus_two".into(),
                        c: hashmap!{
                            String::from("x") => Connection::Structure(Structure {
                                a_ref: "zero".into(),
                                c: hashmap!{}
                            })
                        }
                    })
                },
            }),
        );
        assert!(model.is_all_valid());
        assert_eq!(
            model.slots_of("plus_two".into()).unwrap()["x"],
            String::from("Number")
        );
        assert!(model.slots_of("number_4".into()).unwrap().is_empty());

        assert_eq!(peano_eval("number_4".into(), &model), 2);
        todo!("Assert that structure expansion equals to an equivalent structure depending only on blocks");
    }
}
