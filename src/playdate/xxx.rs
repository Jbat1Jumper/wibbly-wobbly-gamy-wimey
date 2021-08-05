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

    #[derive(Debug)]
    enum InstanceAnACommandError {
        DuringFetch(Entity, FetchError),
        Instancing(Entity, ErrorInstancingAnA),
    }

    impl Command for InstanceAnACommand {
        fn write(self: Box<Self>, world: &mut World) {
            fetch(&self.a_ref, world)
                .map_err(|err| InstanceAnACommandError::DuringFetch(self.root, err))
                .and_then(|a: A| {
                    a.create(self.root, world)
                        .map_err(|err| InstanceAnACommandError::Instancing(self.root, err))
                })
                .unwrap_or_else(|err| {
                    let mut errors = world
                        .get_resource_mut::<Vec<InstanceAnACommandError>>()
                        .unwrap();
                    errors.push(err);
                });
        }
    }

    trait W {
        type Id;

        fn create_an_y(&mut self, y: Y, root: Self::Id) -> Result<(), YCreateError>;
    }

    impl W for World {
        type Id = Entity;
        fn create_an_y(&mut self, y: Y, root: Self::Id) -> Result<(), YCreateError> {
            todo!()
        }
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
