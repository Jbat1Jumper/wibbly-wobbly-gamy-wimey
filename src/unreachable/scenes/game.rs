use glam::f32::*;

use std::time::Duration;

use std::collections::{HashMap, HashSet};

use legion::systems::CommandBuffer;
use legion::*;

#[macro_use]
use crate::common::*;
use crate::physics::*;

mod level_gen;
mod room_gen;
mod room_blueprint_to_world;

#[derive(Clone, Copy, Debug)]
enum ChabonKind {
    Player,
    Blob,
}

#[derive(Clone, Copy, Debug, Default)]
struct JoystickControlledVehicle {
    stick: Vec2,
}

fn prototype_player(w: &mut World) -> Entity {
    w.push((
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
    use room_blueprint_to_world::{AnimatedTile, TileGraphic::*, Tileset};

    let x = Unrestricted;
    let tileset = Tileset {
        pyxel_file: "base.pyxel",
        tile_width: 16,
        tile_height: 16,
        tile_constrains: map!{
            3 => [
                x,              x,              x,
                x,              MustBe(Wall),   MustBe(Wall),
                x,              MustBe(Wall),   MustBe(Ground),
            ],
            4 => [
                x,              x,              x,
                MustBe(Wall),   MustBe(Wall),   MustBe(Wall),
                x,              MustBe(Ground), x,
            ],
            12 => [
                x,              x,              x,
                MustBe(Wall),   MustBe(Wall),   MustBe(Wall),
                x,              MustBe(Ground), x,
            ],
            7 => [
                x,              x,              x,
                x,              MustBe(Ground), x,
                x,              x,              x,
            ],
            2 => [
                x,              x,              x,
                x,              MustBe(Empty),  x,
                x,              x,              x,
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

pub struct GameScene {
}

pub struct CurrentRoom(pub &'static str);

impl GameScene {
    pub fn init(world: &mut World, resources: &mut Resources) -> Schedule {
        println!("Init game scene");
        let font = Font::LiberationMono;

        initialize_base_tileset(resources);
        use room_gen::model::RoomGenerator;
        let mut bp = room_gen::Lvl1RoomGenerator::create("S");
        room_blueprint_to_world::create(&bp, world, resources);

        prototype_player(world);

        let cmds: Vec<RoomCommand> = vec![];
        resources.insert(cmds);
        resources.insert(CurrentRoom("S"));

        Schedule::builder()
            .add_system(update_game_scene_system())
            .add_system(update_chabon_sprites_system())
            .add_system(update_sprites_system_that_should_be_in_common_mod_system())
            .add_system(update_room_system())
            .add_system(move_vehicles_system())
            .add_system(update_joystick_controlled_vehicles_system())
            .build()
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
    #[resource] sprite_sheets: &PyxelFiles,
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

    if frame % 100 == 0 {
        println!("current_animation = {}", sprite.current_animation);
    }
}

#[system(for_each)]
fn update_sprites_system_that_should_be_in_common_mod(
    sprite: &mut Sprite,
    #[resource] LastFrameDuration(duration): &LastFrameDuration,
    #[resource] sprite_sheets: &PyxelFiles,
) {
    let delta = duration.as_secs_f64();
    if let Some(sprite_sheet) = sprite_sheets.0.get(&sprite.pyxel_file) {
        if let Ok(duration) = sprite_sheet.get_animation_duration(&sprite.current_animation) {
            sprite.current_animation_time = (sprite.current_animation_time + delta) % duration;
        }
    }
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
        let direction = match b {
            Button::Down => vec2_down(),
            Button::Up => vec2_up(),
            Button::Left => vec2_left(),
            Button::Right => vec2_right(),
            _ => Vec2::new(0.0, 0.0),
        };

        let polarity = match bs {
            ButtonState::Pressed => 1.0,
            ButtonState::Released => -1.0,
        };

        controller.stick += direction * polarity;
        if controller.stick.distance(Vec2::zero()) > 0.01 {
            controller.stick = Vec2::new(
                // TODO: Change to .clamp() when on rust-1.50
                controller.stick.x.min(1.0).max(-1.0),
                controller.stick.y.min(1.0).max(-1.0),
            );
        }
    }

    vehicle.force += controller.stick * vehicle.speed;
}
