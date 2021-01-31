pub use std::time::Duration;

use ggez;
use glam::f32::Vec2;

use std::env;
use std::path;
use crate::backend::*;

use std::sync::mpsc::{channel, Receiver, Sender};

use std::collections::{HashMap, HashSet};

use legion::*;

use pyxel::Pyxel;


pub use Constrain::*;
pub use Direction::*;
pub use Rotation::*;

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


#[derive(Clone, Copy, Debug)]
pub enum Button {
    Start,
    A,
    B,
    Left,
    Right,
    Up,
    Down,
}

pub trait Scene {
    fn update(&mut self, ctx: &mut Backend, cmd: &mut Sender<SceneCommand>) -> GameResult;
    fn draw(&mut self, ctx: &mut Backend) -> GameResult;
    fn on_input(&mut self, ctx: &mut Backend, button: &Button, state: &ButtonState, cmd: &mut Sender<SceneCommand>) -> GameResult;
}

#[derive(Clone, Copy, Debug)]
pub enum SceneCommand {
    GoTo(SceneRef),
    Exit,
}

#[derive(Clone, Copy, Debug)]
pub struct SceneRef(pub &'static str);

#[derive(Clone, Copy, Debug)]
pub enum ButtonState {
    Pressed,
    Released,
}


#[derive(Clone, Copy, Debug)]
pub enum Constrain<T> {
    Unrestricted,
    MustBe(T),
}

impl<T> Constrain<T>
where
    T: Eq,
{
    pub fn satisfies(&self, item: &T) -> bool {
        match self {
            Unrestricted => true,
            MustBe(something) => something == item,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}
impl Direction {
    pub fn all() -> [Direction; 4] {
        [Up, Left, Down, Right]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Rotation {
    pub fn all() -> [Rotation; 4] {
        [Deg0, Deg90, Deg180, Deg270]
    }
}

pub trait Rotable {
    fn rotate(self, rotation: &Rotation) -> Self;
}

pub trait Flippable {
    fn flip_horizontally(self) -> Self;
}

pub trait GridWalkable {
    fn step(&self, direction: Direction) -> Self;
}

impl GridWalkable for (i32, i32) {
    fn step(&self, direction: Direction) -> Self {
        let (x, y) = *self;
        match direction {
            Up => (x, y - 1),
            Left => (x - 1, y),
            Down => (x, y + 1),
            Right => (x + 1, y),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Position(pub Vec2);

#[derive(Clone, Copy, Debug)]
pub enum Sprite {
    TileRef(usize, TilesetRef),
}

#[derive(Clone, Copy, Debug)]
pub enum TilesetRef {
    PyxelFile(&'static str),
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteTransform {
    pub rotation: Rotation,
    pub flipped: bool,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Text {
    pub string: String,
    pub font: Font,
    pub size: usize,
}

impl Text {
    pub fn new<T: Into<String>>(s: T, font: Font, size: usize) -> Text {
        Text {
            string: s.into(),
            font,
            size,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Font {
    LiberationMono,
}

impl Font {
    pub fn resource_path(&self) -> &'static str {
        match self {
            Font::LiberationMono => "/LiberationMono-Regular.ttf",
        }
    }
}
