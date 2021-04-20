use glam::f32::*;

use std::time::Duration;

use std::collections::{HashMap, HashSet};

use legion::systems::CommandBuffer;
use legion::*;

#[macro_use]
use crate::common::*;
use crate::physics::*;

mod level_gen;
mod room_blueprint_to_world;
mod room_gen;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChabonKind {
    Player,
    Blob,
}

#[derive(Clone, Debug, Default)]
struct JoystickControlledVehicle {
    input_map: HashMap<Button, ButtonState>,
}

impl JoystickControlledVehicle {
    pub fn stick(&self) -> Vec2 {
        let mut stick = Vec2::new(0.0, 0.0);

        for (b, bs) in self.input_map.iter() {
            let direction = match b {
                Button::Down => vec2_down(),
                Button::Up => vec2_up(),
                Button::Left => vec2_left(),
                Button::Right => vec2_right(),
                _ => Vec2::new(0.0, 0.0),
            };
            if *bs == ButtonState::Pressed {
                stick += direction;
            }
        }

        stick.normalize()
    }
}

fn prototype_player(cmd: &mut CommandBuffer) -> Entity {
    cmd.push((
        ChabonKind::Player,
        Position(Vec2::new(30.0, 30.0)),
        Vehicle::default(),
        JoystickControlledVehicle::default(),
        Sprite {
            pyxel_file: "base.pyxel",
            current_animation: "idle".into(),
            current_animation_time: 0.0,
        },
        SpriteTransform::default(),
        RigidBody2D::new(Shape::Circle(3.0), false),
    ))
}

#[rustfmt::skip]
fn initialize_base_tileset(resources: &mut Resources) {
    use room_blueprint_to_world::{AnimatedTile, TileConstrain::*, Tileset};

    let tileset = Tileset {
        pyxel_file: "base.pyxel",
        tile_width: 16,
        tile_height: 16,
        tile_constrains: map!{
            3 => [
                X,       X,       X,
                X,       Wall,    Wall,
                X,       Wall,    Ground,
            ],
            4 => [
                X,       X,       X,
                Wall,    Wall,    Wall,
                X,       Ground,  X,
            ],
            4 => [
                X,       X,       X,
                Wall,    Wall,    Wall,
                X,       Ground,  X,
            ],
            12 => [
                X,       X,       X,
                Solid,   Wall,    Door,
                X,       Ground,  X,
            ],
            27 => [
                X,       X,       X,
                Solid,   Door,    Solid,
                X,       Ground,  X,
            ],
            7 => [
                X,       X,       X,
                X,       Ground,  X,
                X,       X,       X,
            ],
            2 => [
                X,       X,       X,
                X,       Empty,   X,
                X,       X,       X,
            ]
        },
        animations: vec![
            AnimatedTile {
                name: "wall_with_torch",
                intrinsic: true,
                frames: vec![12, 13, 15],
            },
        ],
    };

    resources.insert(tileset);
}

enum RoomInput {
    Frame(Duration),
    Button(Button, ButtonState),
    PlayerEnters(Direction),
}

enum RoomCommand {
    PlayerExits(Direction),
    PlayerDied,
}

////////////////////////////////////////////////////////////////////
// GAME SCENE
////////////////////////////////////////////////////////////////////

pub struct GameScene {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurrentRoom(pub &'static str);

impl GameScene {
    pub fn init(world: &mut World, resources: &mut Resources) -> Schedule {
        println!("Init game scene");
        let font = Font::LiberationMono;

        initialize_base_tileset(resources);
        let cmds: Vec<RoomCommand> = vec![];
        resources.insert(cmds);
        resources.insert(GameSceneState::Initial);
        resources.insert(CurrentRoom("S"));
        resources.insert(LoadRoom(true));

        let dd = dungeon_definition::lvl_1();
        let lvl_gen_state = level_gen::State::new(dd);
        resources.insert(lvl_gen_state);

        let contact_events: Receiver<ContactEvent> =
            (*resources.get::<Receiver<ContactEvent>>().unwrap()).clone();

        println!("Returning schedule");
        Schedule::builder()
            .add_system(create_gizmos_system())
            .add_system(update_game_scene_system())
            .add_system(update_state_transitions_system())
            .add_system(update_chabon_sprites_system())
            .add_system(handle_door_contact_system(contact_events))
            .add_system(update_sprites_system_that_should_be_in_common_mod_system())
            .add_system(update_room_system())
            .add_system(move_vehicles_system())
            .add_system(update_joystick_controlled_vehicles_system())
            .add_system(load_room_system())
            .build()
    }
}

////////////////////////////////////////////////////////////////////
// SYSTEMS
////////////////////////////////////////////////////////////////////

enum GameSceneState {
    Initial,
    EnteringRoom,
    Play,
    ExitingRoom,
}

impl Default for GameSceneState {
    fn default() -> Self {
        GameSceneState::Initial
    }
}

#[system]
fn update_state_transitions(#[resource] state: &mut GameSceneState) {
    use GameSceneState::*;
    // let state = resources.get_or_default::<GameSceneState>();
    match *state {
        Initial => {
            *state = EnteringRoom;
        }
        EnteringRoom => {
            *state = Play;
        }
        Play => {}
        ExitingRoom => {
            *state = EnteringRoom;
        }
    }
}

#[system]
fn update_game_scene(
    #[resource] cmd: &mut Vec<SceneCommand>,
    #[resource] input: &Vec<(Button, ButtonState)>,
) {
    for (button, _state) in input.iter() {
        match button {
            Button::Start => cmd.push(SceneCommand::Exit),
            _ => (),
        }
    }
}

#[system(for_each)]
fn update_chabon_sprites(
    chabon: &ChabonKind,
    vehicle: &Vehicle,
    sprite: &mut Sprite,
    #[resource] LastFrameDuration(duration): &LastFrameDuration,
    #[resource] CurrentFrame(frame): &CurrentFrame,
) {
    let dir = vec![
        ("left", vec2_left()),
        ("right", vec2_right()),
        ("up", vec2_up()),
        ("down", vec2_down()),
    ]
    .into_iter()
    .map(|(name, target_dir)| (name, target_dir.distance(vehicle.direction)))
    .min_by_key(|(_name, distance)| (distance * 100.0) as i32)
    .unwrap()
    .0;

    let state = if vehicle.force.length_squared() > 0.01 {
        "walk"
    } else {
        "idle"
    };

    let new_animation = format!("{}_{}", dir, state);
    if sprite.current_animation != new_animation {
        sprite.current_animation = new_animation;
        sprite.current_animation_time = 0.0;
    }
}


use crate::physics::ContactEvent;
use crossbeam_channel::Receiver;
use legion::world::SubWorld;
pub mod dungeon_definition;

use dungeon_definition::DungeonDefinition;

struct LoadRoom(bool);
struct LastDoorUsed(usize);

#[system]
#[read_component(room_gen::model::Tile)]
#[read_component(ChabonKind)]
fn load_room(
    world: &SubWorld,
    cmd: &mut CommandBuffer,
    #[resource] tileset: &room_blueprint_to_world::Tileset,
    #[resource] load_room: &mut LoadRoom,
    #[resource] lvl_gen_state: &mut level_gen::State<DungeonDefinition>,
) {
    if let LoadRoom(true) = load_room {
        // Delete old room entities
        let mut query = <(Entity, &ChabonKind)>::query();
        for (entity, _) in query.iter(world) {
            cmd.remove(*entity);
        }
        let mut query = <(Entity, &room_gen::model::Tile)>::query();
        for (entity, _) in query.iter(world) {
            cmd.remove(*entity);
        }

        // Create new room
        use room_gen::model::RoomGenerator;
        let bp = lvl_gen_state.definition.create(lvl_gen_state.current_room);
        room_blueprint_to_world::create(&bp, cmd, tileset);
        prototype_player(cmd);

        *load_room = LoadRoom(false);
    }
}

use room_gen::model::Tile;
// PLS TODO: Refactor this, it looks really dirty
#[system]
#[read_component(room_gen::model::Tile)]
#[read_component(ChabonKind)]
fn handle_door_contact(
    world: &SubWorld,
    #[state] contact_events: &Receiver<ContactEvent>,
    #[resource] current_room: &mut CurrentRoom,
    #[resource] load_room: &mut LoadRoom,
    #[resource] lvl_gen_state: &mut level_gen::State<DungeonDefinition>,
) {
    for e in contact_events.try_iter() {
        match e {
            ContactEvent::Started(this, that) => {
                if let Some(ChabonKind::Player) = world.try_get_cloned(this) {
                    if let Some(Tile::Door(dn)) = world.try_get_cloned(that) {
                        println!("Before {:?}", current_room);
                        println!("Going through door!");
                        let res = lvl_gen_state.step(dn);
                        *current_room = CurrentRoom(lvl_gen_state.current_room);
                        println!("Result {:?}", res);
                        println!("After {:?}", current_room);
                        // println!("State {:#?}", lvl_gen_state);
                        *load_room = LoadRoom(true);
                    }
                }
            }
            _ => {}
        }
    }
    // TODO: Do something
}


#[system]
fn update_room(#[resource] command_buffer: &Vec<RoomCommand>) {
    // TODO: Do something
}

#[system(for_each)]
fn move_vehicles(
    position: &mut Position,
    vehicle: &mut Vehicle,
    rigidbody: Option<&RigidBody2D>,
    #[resource] LastFrameDuration(duration): &LastFrameDuration,
    #[resource] PhysicsResources { bodies, .. }: &mut PhysicsResources,
) {
    if vehicle.force.length_squared() > 0.01 {
        let new_p = position.0 + vehicle.force * duration.as_secs_f32();
        if let Some(rigidbody) = rigidbody {
            if let Some((rbh, _ch)) = rigidbody.handles {
                let rb = bodies
                    .get_mut(rbh)
                    .expect("RigidBody not found for handle moving vehicles");
                let f =
                    rapier2d::na::Vector2::new(vehicle.force.x * 100.0, vehicle.force.y * 100.0);
                rb.apply_force(f, true);
            }
        } else {
            position.0 = new_p;
        }
        vehicle.direction = vehicle.force.normalize();
    }
    vehicle.force = Vec2::new(0.0, 0.0);
}

#[system(for_each)]
fn update_joystick_controlled_vehicles(
    controller: &mut JoystickControlledVehicle,
    vehicle: &mut Vehicle,
    #[resource] input: &Vec<(Button, ButtonState)>,
) {
    for (b, bs) in input.iter() {
        controller.input_map.insert(*b, *bs);
    }

    vehicle.force += controller.stick() * vehicle.speed;
}
