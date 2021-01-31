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
use crate::common::*;

#[derive(Clone, Copy, Debug)]
enum Sprite {
    TileRef(usize, TilesetRef),
}

#[derive(Clone, Copy, Debug)]
struct SpriteTransform {
    rotation: Rotation,
    flipped: bool,
}

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

fn get_tile_sprite_and_transform(
    pos: (i32, i32),
    tile_map: &TileMap,
    tileset: &Tileset,
) -> Option<(Sprite, SpriteTransform)> {
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

                return Some((
                    Sprite::TileRef(*id, tileset.reference()),
                    SpriteTransform {
                        rotation: *rotation,
                        flipped: *flipped,
                    },
                ));
            }
        }
    }
    None
}

#[derive(Clone, Copy, Debug)]
struct Frame(u32);

#[derive(Clone, Copy, Debug)]
enum TilesetRef {
    PyxelFile(&'static str),
}

struct Tileset {
    pyxel_file: &'static str,
    tiles: HashMap<usize, [Constrain<Tile>; 9]>,
    animations: Vec<AnimatedTile>,
}

struct AnimatedTile {
    name: &'static str,
    /// Indicates that any tile that matches a frame should be animated
    intrinsic: bool,
    frames: Vec<usize>,
}

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

#[rustfmt::skip]
fn base_tileset() -> Tileset {
    use Tile::*;
    let x = Unrestricted;
    Tileset {
        pyxel_file: "base.pyxel",
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
        TilesetRef::PyxelFile(self.pyxel_file)
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
    fn populate(&mut self, room: &mut RoomBlueprint, params: &RoomParams, rng: &mut Rng);
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

    fn populate(&mut self, room: &mut RoomBlueprint, params: &RoomParams, rng: &mut Rng) {
        room.tile_map.set_at((0, 0), Tile::Ground);
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

impl Room {
    fn draw(&self, ctx: &mut Backend) -> GameResult {
        // query and draw room entities:
        // - draw tiles
        // - draw shadows?
        // - draw entities
        // - draw effects
        Ok(())
    }

    fn update(&mut self, event: RoomInput, cmd: Sender<RoomCommand>) {
        // all room systems
        // - player enters/exits handling
    }
}

fn room_from_blueprint(blueprint: RoomBlueprint, tileset: &Tileset) -> Room {
    let mut world = World::default();
    world.extend(
        iter_positions(blueprint.size)
            .iter()
            .filter_map(|pos| get_tile_sprite_and_transform(*pos, &blueprint.tile_map, tileset)),
    );

    Room { world }
}

pub struct GameScene {
    rooms: Vec<RoomEntry>,
    current_entry: isize,
}

struct RoomEntry {
    position: (isize, isize),
    room: Room,
    // this could be really interesing with ttl instead of age.
    // if ttl runs to 0 that means that the room was not visited for
    // enough time to discard it and also to randomize stuff in it.
    // the randomized stuff can be enhanced as the player unlocks
    // more stuff.
    age: usize,
}

impl GameScene {
    pub fn new() -> GameScene {
        GameScene {
            rooms: vec![RoomEntry {
                room: Self::initial_room(),
                position: (0, 0),
                age: 0,
            }],
            current_entry: 0,
        }
    }

    fn initial_room() -> Room {
        let bp = SimpleRoomCreator::create_room(
            &RoomParams {
                connection_constrains: map! {
                    Up => MustBe(Connection::Common)
                },
            },
            &mut Rng,
        );
        room_from_blueprint(bp, &base_tileset())
    }
}

impl Scene for GameScene {
    fn update(&mut self, ctx: &mut Backend, cmd: &mut Sender<SceneCommand>) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Backend) -> GameResult {
        Ok(())
    }
    fn on_input(
        &mut self,
        ctx: &mut Backend,
        button: &Button,
        state: &ButtonState,
        cmd: &mut Sender<SceneCommand>,
    ) -> GameResult {
        Ok(())
    }
}
