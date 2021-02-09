use ggez;
use glam::f32::Vec2;

use std::env;
use std::path;
use std::time::Duration;

use std::sync::mpsc::{channel, Receiver, Sender};

use std::collections::{HashMap, HashSet};

use legion::*;

use pyxel::Pyxel;

use crate::backend::*;
#[macro_use]
use crate::common::*;
use crate::pubsub::{Publication, Subscription};

#[derive(Clone, Debug)]
struct TileMap {
    tiles: Vec<Vec<Tile>>,
    size: (usize, usize),
}

impl TileMap {
    fn new(size: (usize, usize)) -> TileMap {
        TileMap {
            size,
            tiles: std::iter::repeat_with(|| std::iter::repeat(Tile::Empty).take(size.0).collect())
                .take(size.1)
                .collect(),
        }
    }

    fn at(&self, pos: (i32, i32)) -> Tile {
        if self.in_bounds(pos) {
            self.tiles[pos.1 as usize][pos.0 as usize]
        } else {
            Tile::Empty
        }
    }

    fn set_at(&mut self, pos: (i32, i32), tile: Tile) {
        if self.in_bounds(pos) {
            self.tiles[pos.1 as usize][pos.0 as usize] = tile;
        }
    }

    fn in_bounds(&self, pos: (i32, i32)) -> bool {
        pos.0 >= 0 && pos.0 < self.size.0 as i32 && pos.1 >= 0 && pos.1 < self.size.1 as i32
    }

    #[rustfmt::skip]
    fn neighborhood(&self, pos: (i32, i32)) -> [Tile; 9] {
        [
            self.at(pos.step(Left).step(Up)),   self.at(pos.step(Up)),   self.at(pos.step(Right).step(Up)),
            self.at(pos.step(Left)),            self.at(pos),            self.at(pos.step(Right)),
            self.at(pos.step(Left).step(Down)), self.at(pos.step(Down)), self.at(pos.step(Right).step(Down)),
        ]
    }
}

impl<T> Rotable for [T; 9]
where
    T: Copy,
{
    #[rustfmt::skip]
    fn rotate(self, rotation: &Rotation) -> [T; 9] {
        match rotation {
            Deg0 => self,
            Deg90 => [
                self[6], self[3], self[0],
                self[7], self[4], self[1],
                self[8], self[5], self[2],
            ],
            Deg180 => [
                self[8], self[7], self[6],
                self[5], self[4], self[3],
                self[2], self[1], self[0],
            ],
            Deg270 => [
                self[2], self[5], self[8],
                self[1], self[4], self[7],
                self[0], self[3], self[6],
            ],
        }
    }
}

impl<T> Flippable for [T; 9]
where
    T: Copy,
{
    #[rustfmt::skip]
    fn flip_horizontally(self) -> [T; 9] {
        [
            self[2], self[1], self[0],
            self[5], self[4], self[3],
            self[8], self[7], self[6],
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Empty,
    Ground,
    Wall,
    Door(DoorKind, DoorState, Direction),
}

fn get_tile_components(
    pos: (i32, i32),
    tile_map: &TileMap,
    tileset: &Tileset,
) -> Option<(TileRef, SpriteTransform, Position)> {
    let nh = tile_map.neighborhood(pos);

    for flipped in &[false, true] {
        for rotation in &Rotation::all() {
            for (id, constrains) in tileset.tiles.iter() {
                let constrains = if *flipped {
                    constrains.clone().flip_horizontally()
                } else {
                    constrains.clone()
                };
                let constrains = constrains.clone().rotate(rotation);
                let fits = constrains
                    .iter()
                    .zip(nh.iter())
                    .all(|(constrain, tile)| constrain.satisfies(tile));

                if fits {
                    return Some((
                        TileRef(*id, tileset.reference()),
                        SpriteTransform {
                            rotation: *rotation,
                            flipped: *flipped,
                        },
                        Position(Vec2::new(
                            pos.0 as f32 * tileset.tile_width as f32,
                            pos.1 as f32 * tileset.tile_height as f32,
                        )),
                    ));
                }
            }
        }
    }
    None
}

struct Tileset {
    pyxel_file: &'static str,
    tiles: HashMap<usize, [Constrain<Tile>; 9]>,
    animations: Vec<AnimatedTile>,
    tile_width: usize,
    tile_height: usize,
}

struct AnimatedTile {
    name: &'static str,
    /// Indicates that any tile that matches a frame should be animated
    intrinsic: bool,
    frames: Vec<usize>,
}

#[rustfmt::skip]
fn base_tileset() -> Tileset {
    use Tile::*;
    let x = Unrestricted;
    Tileset {
        pyxel_file: "base.pyxel",
        tile_width: 16,
        tile_height: 16,
        tiles: map!{
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
                x,              MustBe(Empty), x,
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
    }
}

impl Tileset {
    fn reference(&self) -> TilesetRef {
        TilesetRef {
            pyxel_file: self.pyxel_file,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DoorState {
    Open,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DoorKind {
    Known,
    Unknown,
    Dark,
}

#[derive(Clone, Debug)]
struct Rng;

#[derive(Clone, Debug)]
struct RoomParams {
    connection_constrains: HashMap<Direction, Constrain<Connection>>,
}

#[derive(Clone, Debug)]
struct RoomBlueprint {
    size: (i32, i32),
    tile_map: TileMap,
    connections: HashMap<Direction, Connection>,
}

fn iter_positions(size: (i32, i32)) -> Vec<(i32, i32)> {
    (0..size.1)
        .map(|y| (0..size.0).map(move |x| (x, y)))
        .flatten()
        .collect()
}

#[derive(Clone, Copy, Debug)]
enum Connection {
    Common,
    Dark,
    NotConnected,
}

trait RoomCreator {
    fn create_room(params: &RoomParams, rng: &mut Rng) -> RoomBlueprint;
    fn populate(room: &mut RoomBlueprint, params: &RoomParams, rng: &mut Rng);
}

struct SimpleRoomCreator;

impl RoomCreator for SimpleRoomCreator {
    fn create_room(params: &RoomParams, rng: &mut Rng) -> RoomBlueprint {
        let mut connections = HashMap::new();

        for dir in &Direction::all() {
            connections.insert(
                *dir,
                match params.connection_constrains.get(dir) {
                    Some(constrain) => match constrain {
                        Unrestricted => Connection::Common,
                        MustBe(con) => *con,
                    },
                    None => Connection::Common,
                },
            );
        }
        RoomBlueprint {
            connections,
            size: (8, 5),
            tile_map: TileMap::new((8, 5)),
        }
    }

    fn populate(room: &mut RoomBlueprint, params: &RoomParams, rng: &mut Rng) {
        for x in (0..room.size.0) {
            for y in (0..room.size.1) {
                room.tile_map.set_at((x, y), Tile::Ground);
            }
        }
        for x in (0..room.size.0) {
            room.tile_map.set_at((x, 0), Tile::Wall);
        }
        for x in (0..room.size.0) {
            room.tile_map.set_at((x, room.size.1 - 1), Tile::Wall);
        }
        for y in (0..room.size.1) {
            room.tile_map.set_at((0, y), Tile::Wall);
        }
        for y in (0..room.size.1) {
            room.tile_map.set_at((room.size.0 - 1, y), Tile::Wall);
        }
    }
}

struct Room {
    world: World,
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

fn populate_room_from_blueprint(world: &mut World, blueprint: &RoomBlueprint, tileset: &Tileset) {
    world.extend(
        iter_positions(blueprint.size)
            .iter()
            .filter_map(|pos| get_tile_components(*pos, &blueprint.tile_map, tileset)),
    );
    world.push((
        Sprite {
            pyxel_file: "base.pyxel",
            current_animation: "idle".into(),
            current_animation_time: 0.0,
        },
        SpriteTransform::default(),
        Position(Vec2::new(30.0, 30.0)),
    ));
}

pub struct GameScene {
    rooms: Vec<RoomEntry>,
    current_entry: usize,
    cmd_bus: Receiver<RoomCommand>,
    cmd: Sender<RoomCommand>,
}

struct RoomEntry {
    position: (isize, isize),
    room: RoomBlueprint,
    // this could be really interesing with ttl instead of age.
    // if ttl runs to 0 that means that the room was not visited for
    // enough time to discard it and also to randomize stuff in it.
    // the randomized stuff can be enhanced as the player unlocks
    // more stuff.
    age: usize,
}


struct RoomsEntries(pub Vec<RoomEntry>);
pub struct CurrentRoom(pub usize);

#[system(for_each)]
fn update_sprites(
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
fn update_room(
    #[resource] command_buffer: &Vec<RoomCommand>,
) {
    // TODO: Do something
}



impl GameScene {
    pub fn init(world: &mut World, resources: &mut Resources) -> Schedule {
        let font = Font::LiberationMono;

        let params = RoomParams {
            connection_constrains: map! {
                Up => MustBe(Connection::Common)
            },
        };
        let mut bp = SimpleRoomCreator::create_room(&params, &mut Rng);
        SimpleRoomCreator::populate(&mut bp, &params, &mut Rng);
        populate_room_from_blueprint(world, &bp, &base_tileset());

        let cmds: Vec<RoomCommand> = vec![];
        resources.insert(cmds);
        resources.insert(CurrentRoom(0));
        resources.insert(vec![RoomEntry {
            room: bp,
            position: (0, 0),
            age: 0,
        }]);

        Schedule::builder()
            .add_system(update_sprites_system())
            .add_system(update_room_system())
            .build()
    }
}
