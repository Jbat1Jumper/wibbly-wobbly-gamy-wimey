use bevy::prelude::*;
pub use glam_ext::*;
pub use known_fonts::*;
pub mod known_fonts;

pub mod glam_ext {
    use bevy::prelude::*;
    pub fn vec2_left() -> Vec2 {
        Vec2::new(-1.0, 0.0)
    }
    pub fn vec2_right() -> Vec2 {
        Vec2::new(1.0, 0.0)
    }
    pub fn vec2_up() -> Vec2 {
        Vec2::new(0.0, 1.0)
    }
    pub fn vec2_down() -> Vec2 {
        Vec2::new(0.0, -1.0)
    }
}

pub use Dir::*;
pub use Rot::*;

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

pub fn create_gizmos(commands: Commands, query: Query<(Entity, &Transform)>) {
    panic!("Not implemented, see bevy_lyon/bevy_prototype_lyon");
}

#[derive(Clone, Copy, Debug)]
pub struct Vehicle {
    pub direction: Vec2,
    pub speed: f32,
    // direction: f64,
    // force: f64,
}

impl Default for Vehicle {
    fn default() -> Self {
        Vehicle {
            direction: Vec2::new(1.0, 0.0),
            speed: 100.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Dir {
    Up,
    Left,
    Down,
    Right,
}

impl Dir {
    pub fn all() -> [Dir; 4] {
        [Up, Left, Down, Right]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rot {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Rot {
    pub fn all() -> [Rot; 4] {
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

impl Into<Quat> for Rot {
    fn into(self) -> Quat {
        Quat::from_rotation_z(-self.radians())
    }
}

pub trait Rotable {
    fn rotate(self, rotation: &Rot) -> Self;
}

pub trait Flippable {
    fn flip_horizontally(self) -> Self;
}

impl<T> Rotable for [T; 9]
where
    T: Copy,
{
    #[rustfmt::skip]
    fn rotate(self, rotation: &Rot) -> [T; 9] {
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

pub trait GridWalkable {
    fn step(&self, direction: Dir) -> Self;
}

impl GridWalkable for (i32, i32) {
    fn step(&self, direction: Dir) -> Self {
        let (x, y) = *self;
        match direction {
            Up => (x, y - 1),
            Left => (x - 1, y),
            Down => (x, y + 1),
            Right => (x + 1, y),
        }
    }
}
