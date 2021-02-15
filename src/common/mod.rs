pub use std::time::Duration;

use glam::f32::Vec2;

use std::env;
use std::path;

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

pub struct LastFrameDuration(pub Duration);

pub struct MustQuit(pub bool);

impl Default for MustQuit {
    fn default() -> Self {
        MustQuit(false)
    }
}

pub struct PyxelFiles(pub HashMap<&'static str, pyxel::Pyxel>);

pub struct CurrentFPS(pub f64);

pub struct CurrentFrame(pub usize);


// pub trait Scene {
//     fn update(&mut self, ctx: &mut GgezBackend, cmd: &mut Sender<SceneCommand>) -> GameResult;
//     fn draw(&mut self, ctx: &mut GgezBackend) -> GameResult;
//     fn on_input(
//         &mut self,
//         ctx: &mut GgezBackend,
//         button: &Button,
//         state: &ButtonState,
//         cmd: &mut Sender<SceneCommand>,
//     ) -> GameResult;
// }

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

    pub fn radians(&self) -> f32 {
        use std::f32::consts::PI;
        match self {
            Deg0 => 0.0,
            Deg90 => 0.5 * PI,
            Deg180 => PI,
            Deg270 => 1.5 * PI,
        }
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

#[derive(Clone, Debug)]
pub struct Sprite {
    pub pyxel_file: &'static str,
    pub current_animation: AnimationId,
    pub current_animation_time: f64,
}

pub trait SpriteSheet {
    fn get_size(&self) -> (usize, usize);
    fn get_animations(&self) -> Vec<AnimationId>;
    fn get_layers(&self) -> Vec<LayerId>;
    fn get_animation_frames(
        &self,
        animation: &AnimationId,
    ) -> Result<Vec<(FrameId, f64)>, String>;

    fn get_animation_duration(
        &self,
        animation: &AnimationId,
    ) -> Result<f64, String> {
        Ok(self.get_animation_frames(animation)?.iter().map(|(_, d)| d).sum())
    }

    fn get_frame_data_in_rgba8(
        &self,
        frame: &FrameId,
        layer: &LayerId,
    ) -> Result<Vec<u8>, String>;

    fn get_frame_at(&self, animation: &AnimationId, at_time: f64) -> Result<FrameId, String> {
        let mut t = 0.0;
        for (frame, duration) in self.get_animation_frames(animation)?.iter() {
            if t + duration > at_time {
                return Ok(*frame);
            }
            t += duration;
        }
        Err(format!("Time {} out of bounds in animation {}", at_time, animation))
    }
}

pub type AnimationId = String;
pub type FrameId = u32;
pub type LayerId = String;

impl SpriteSheet for Pyxel {

    fn get_size(&self) -> (usize, usize) {
        (
            self.canvas().tile_width().into(),
            self.canvas().tile_height().into(),
        )
    }

    fn get_animations(&self) -> Vec<AnimationId> {
        self.animations()
            .iter()
            .map(pyxel::Animation::name)
            .cloned()
            .collect()
    }

    fn get_layers(&self) -> Vec<LayerId> {
        self.canvas()
            .layers()
            .iter()
            .map(pyxel::Layer::name)
            .cloned()
            .collect()
    }

    fn get_animation_frames(
        &self,
        animation: &AnimationId,
    ) -> Result<Vec<(FrameId, f64)>, String> {
        let animation = self
            .animations()
            .iter()
            .find(|a| a.name() == animation)
            .ok_or(format!("No animation found: {}", animation))?;

        let b = animation.base_tile();
        let r = animation
            .frame_duration_multipliers()
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, duration)| (b as u32 + i as u32, duration))
            .collect();
        Ok(r)
    }

    fn get_frame_data_in_rgba8(
        &self,
        frame_id: &FrameId,
        layer_id: &LayerId,
    ) -> Result<Vec<u8>, String> {
        let frame = *frame_id;
        if frame as i32 >= self.canvas().width() * self.canvas().height() {
            return Err(format!("Frame {} is out of bounds", frame));
        }

        let layer = self
            .canvas()
            .layers()
            .iter()
            .find(|l| l.name() == layer_id)
            .ok_or(format!("No layer found: {}", layer_id))?;

        let tile_width = self.canvas().tile_width() as u32;
        let tile_height = self.canvas().tile_height() as u32;
        let width = self.canvas().width() as u32 / tile_width;
        let (x, y) = (
            frame % width,
            frame / width,
        );

        use image::GenericImageView;

        //panic!(
        //    "tile_width: {}, tile_height: {}, xy: {:?}, layer: {}, frame: {}", 
        //    tile_width, tile_height, (x, y), layer_id, frame);

        Ok(layer
            .image()
            .to_rgba()
            .view(x * tile_width, y * tile_height, tile_width, tile_height)
            .pixels()
            // https://raw.githubusercontent.com/rochacbruno/rust_memes/master/img/lisa.jpg
            .map(|p| Vec::from(p.2.0))  
            .flatten()
            .collect())
    }
}

pub struct TileRef(pub usize, pub TilesetRef);

// This could be an enum again if there are different tileset file types.  For instance: A pyxel
// file, a plain image file, an array of rgba8 images or an external url.
#[derive(Clone, Copy, Debug)]
pub struct TilesetRef {
    pub pyxel_file: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteTransform {
    pub rotation: Rotation,
    pub flipped: bool,
}
impl Default for SpriteTransform {
    fn default() -> SpriteTransform {
        SpriteTransform {
            rotation: Deg0,
            flipped: false,
        }
    }
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
    pub fn truetype_font_bytes(&self) -> &[u8] {
        match self {
            Font::LiberationMono => include_bytes!("resources/LiberationMono-Regular.ttf"),
        }
    }
}
