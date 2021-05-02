use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use heron::prelude::*;
use std::collections::HashMap;
use crate::pyxel_plugin::PyxelSprite;
use crate::unreachable::scenes::UnScene;

use crate::common::*;

pub mod dungeon_definition;
mod level_gen;
mod room_blueprint_to_world;
mod room_gen;

use dungeon_definition::DungeonDefinition;
use room_gen::model::Tile;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChabonKind {
    Player,
    Blob,
}

#[derive(Clone, Debug, Default)]
struct JoystickControlledVehicle {
    input_map: HashMap<Dir, bool>,
}

impl JoystickControlledVehicle {
    pub fn stick(&self) -> Vec2 {
        let mut stick = Vec2::new(0.0, 0.0);

        for (b, bs) in self.input_map.iter() {
            let direction = match b {
                Dir::Down => vec2_down(),
                Dir::Up => vec2_up(),
                Dir::Left => vec2_left(),
                Dir::Right => vec2_right(),
            };
            if *bs {
                stick += direction;
            }
        }

        stick.normalize()
    }
}

fn prototype_player(commands: &mut Commands) {
    commands.spawn_bundle((
        ChabonKind::Player,
        Transform {
            translation: Vec3::new(30., 30., 0.),
            ..Default::default()
        },
        Vehicle::default(),
        JoystickControlledVehicle::default(),
        PyxelSprite {
            pyxel_file: "base.pyxel",
            current_animation: "right_idle".into(),
            current_animation_time: 0.0,
        },
        Body::Sphere { radius: 3. },
        BodyType::Dynamic,
    ));
}

#[rustfmt::skip]
fn base_tileset() -> room_blueprint_to_world::Tileset {
    use room_blueprint_to_world::{AnimatedTile, TileConstrain::*, Tileset};

    Tileset {
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
    }
}

////////////////////////////////////////////////////////////////////
// GAME SCENE
////////////////////////////////////////////////////////////////////

pub struct GameScene;

#[derive(Clone, Debug)]
pub struct GameState {
    current_room: &'static str,
    load_room: bool,
    lvl_gen: level_gen::State<DungeonDefinition>,
    // last_door_used: usize,
}

impl Plugin for GameScene {
    fn build(&self, application: &mut AppBuilder) {
        application
            .insert_resource(base_tileset())
            .insert_resource(GameState {
                current_room: "S",
                load_room: true,
                lvl_gen: level_gen::State::new(dungeon_definition::lvl_1()),
            })
            .add_system_set(SystemSet::on_enter(UnScene::Game).with_system(enter.system()))
            .add_system_set(
                SystemSet::on_update(UnScene::Game)
                    .with_system(update_game_scene.system())
                    .with_system(update_chabon_sprites.system())
                    .with_system(handle_door_contact.system())
                    .with_system(update_joystick_controlled_vehicles.system())
                    .with_system(load_room.system()),
            )
            .add_physics_system(move_vehicles.system())
            .add_system_set(SystemSet::on_exit(UnScene::Game).with_system(exit.system()));
    }
}

fn enter() {}

fn exit() {}

////////////////////////////////////////////////////////////////////
// SYSTEMS
////////////////////////////////////////////////////////////////////

fn update_game_scene(keyboard: Res<Input<KeyCode>>, mut scene: ResMut<State<UnScene>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        scene.set(UnScene::Exit).expect("Failed to set UnScene::Exit");
    }
}

fn update_chabon_sprites(mut query: Query<(&ChabonKind, &Vehicle, &mut PyxelSprite)>, _time: Res<Time>) {
    for (_chabon, vehicle, mut sprite) in query.iter_mut() {
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

        let state = if vehicle.speed > 0.01 {
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
}

fn load_room(
    mut commands: Commands,
    query: Query<Entity, Or<(With<ChabonKind>, With<Tile>)>>,
    tileset: Res<room_blueprint_to_world::Tileset>,
    mut state: ResMut<GameState>,
) {
    if state.load_room {
        // Delete old room entities
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }

        // Create new room
        use room_gen::model::RoomGenerator;
        let bp = state.lvl_gen.definition.create(state.lvl_gen.current_room);
        room_blueprint_to_world::create(&bp, &mut commands, &tileset);
        prototype_player(&mut commands);

        state.load_room = false;
    }
}

fn handle_door_contact(
    mut contact_events: EventReader<CollisionEvent>,
    query_tiles: Query<&Tile>,
    query_chabon: Query<&ChabonKind>,
    mut state: ResMut<GameState>,
) {
    for e in contact_events.iter() {
        match e {
            CollisionEvent::Started(x, y) => for (this, that) in &[(*x, *y), (*y, *x)] {
                if let Ok(ChabonKind::Player) = query_chabon.get(*this) {
                    if let Ok(Tile::Door(dn)) = query_tiles.get(*that) {
                        println!("Before {:?}", state.current_room);
                        println!("Going through door!");
                        let res = state.lvl_gen.step(*dn);
                        state.current_room = state.lvl_gen.current_room;
                        println!("Result {:?}", res);
                        println!("After {:?}", state.current_room);
                        // println!("State {:#?}", state.lvl_gen);
                        state.load_room = true;
                    }
                }
            }
            _ => {}
        }
    }
}

fn move_vehicles(mut query: Query<(&Vehicle, &mut Velocity)>, _time: Res<Time>) {
    for (vehicle, mut velocity) in query.iter_mut() {
        if vehicle.speed > 0.01 {
            velocity.linear = (vehicle.speed * vehicle.direction).extend(0.);
        } else {
            // TODO: Try turn this on and off to see linear damping
            velocity.linear = Vec3::ZERO;
        }
    }
}

fn update_joystick_controlled_vehicles(
    mut query: Query<(&mut Vehicle, &mut JoystickControlledVehicle)>,
    mut keyboard_events: EventReader<KeyboardInput>,
) {
    for (mut vehicle, mut controller) in query.iter_mut() {
        for e in keyboard_events.iter() {
            if let Some(dir) = e.key_code.map(keycode_to_dir).flatten() {
                controller.input_map.insert(dir, e.state.is_pressed());
            }
        }
        vehicle.direction = controller.stick().normalize();
        vehicle.speed = controller.stick().length();
    }
}

fn keycode_to_dir(kc: KeyCode) -> Option<Dir> {
    Some(match kc {
        KeyCode::A => Dir::Left,
        KeyCode::S => Dir::Down,
        KeyCode::D => Dir::Right,
        KeyCode::W => Dir::Up,
        _ => return None,
    })
}
