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
    use bevy::prelude::*;
    use std::collections::HashMap;

    type ARef = String;
    enum A {
        X(Box<dyn X>),
        Y(Y),
    }

    #[derive(Clone)]
    struct Y {
        a_ref: ARef,
        c: HashMap<SName, C>,
    }

    type SName = String;
    type SKind = String;

    #[derive(Clone)]
    struct S {
        name: SName,
        kind: SKind,
    }

    impl S {
        fn create(&self, root: Entity, world: &mut World) {
            world.entity_mut(root).insert(self.clone());
        }
    }

    type XCreateError = String;

    trait X {
        fn create(&self, root: Entity, world: &mut World) -> Result<(), XCreateError>;
    }

    enum ACreateError {
        X(XCreateError),
        Y(YCreateError),
    }

    impl A {
        fn create(&self, root: Entity, world: &mut World) -> Result<(), ACreateError> {
            match self {
                A::X(x) => x.create(root, world).map_err(ACreateError::X),
                A::Y(y) => y.create(root, world).map_err(ACreateError::Y),
            }
        }
    }

    enum YCreateError {
        Fetch(FetchError),
        GetSs(GetSsError),
        CreatingRoot(Box<ACreateError>),
        NoCforS(S),
        CreatingC(S, Entity, Box<Self>),
        YFullfillS(Y, S),
        SFullfillS(S, S),
    }

    impl Y {
        fn create(&self, root: Entity, world: &mut World) -> Result<(), YCreateError> {
            let a = fetch(&self.a_ref).map_err(YCreateError::Fetch)?;
            world.entity_mut(root).insert(C::Y(self.clone()));
            a.create(root, world)
                .map_err(|err| YCreateError::CreatingRoot(Box::new(err)))?;
            let ss = get_ss(root, world).map_err(YCreateError::GetSs)?;
            for (s_entity, s) in ss {
                match self.c.get(&s.name) {
                    Some(C::Y(y)) => {
                        if !y_fullfills_s(y, &s.kind) {
                            return Err(YCreateError::YFullfillS(y.clone(), s));
                        }
                        y.create(s_entity, world).map_err(|err| {
                            YCreateError::CreatingC(s, s_entity, Box::new(err))
                        })?;
                    }
                    Some(C::S(c_s)) => {
                        if !s_fullfills_s(&c_s.kind, &s.kind) {
                            return Err(YCreateError::SFullfillS(c_s.clone(), s.clone()));
                        }
                        c_s.create(s_entity, world);
                    },
                    None => return Err(YCreateError::NoCforS(s)),
                }
            }
            Ok(())
        }
    }

    #[derive(Clone)]
    enum C {
        Y(Y),
        S(S),
    }


    struct FetchError;
    fn fetch(a_ref: &ARef) -> Result<A, FetchError> {
        todo!()
    }

    struct GetSsError;
    fn get_ss(root: Entity, world: &World) -> Result<Vec<(Entity, S)>, GetSsError> {
        todo!()
    }

    struct SFullfilmentError;
    fn y_fullfills_s(y: &Y, s_kind: &SKind) -> bool {
        todo!()
    }
    fn s_fullfills_s(c_s_kind: &SKind, s_kind: &SKind) -> bool {
        todo!()
    }
}
