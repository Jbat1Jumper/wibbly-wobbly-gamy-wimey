use super::room_gen::model::{RoomBlueprint, Tile};
use crate::common::*;
use crate::plain_simple_physics::*;
use crate::pyxel_plugin::PyxelTile;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileConstrain {
    X,
    Empty,
    Ground,
    Wall,
    Door,
    Solid,
}

impl TileConstrain {
    pub fn satisfies(&self, tile: Tile) -> bool {
        match self {
            TileConstrain::X => true,
            TileConstrain::Empty => tile == Tile::Empty,
            TileConstrain::Ground => tile == Tile::Ground,
            TileConstrain::Wall => tile == Tile::Wall,
            TileConstrain::Door => {
                if let Tile::Door(_) = tile {
                    true
                } else {
                    false
                }
            }
            TileConstrain::Solid => match tile {
                Tile::Door(_) | Tile::Wall => true,
                _ => false,
            },
        }
    }
}

pub struct Tileset {
    pub pyxel_file: &'static str,
    pub tile_constrains: HashMap<usize, [TileConstrain; 9]>,
    pub animations: Vec<AnimatedTile>,
    pub tile_width: usize,
    pub tile_height: usize,
}

pub struct AnimatedTile {
    pub name: &'static str,
    /// Indicates that any tile that matches a frame should be animated
    pub intrinsic: bool,
    pub frames: Vec<usize>,
}

fn get_tile_components(
    pos: (i32, i32),
    blueprint: &RoomBlueprint,
    tileset: &Tileset,
) -> Option<(Tile, PyxelTile, Transform)> {
    let nh = blueprint.tile_neighborhood(pos);

    for flipped in &[false, true] {
        for rotation in &Rot::all() {
            for (id, constrains) in tileset.tile_constrains.iter() {
                let constrains = if *flipped {
                    constrains.clone().flip_horizontally()
                } else {
                    constrains.clone()
                };
                let constrains = constrains.clone().rotate(rotation);
                let fits = constrains
                    .iter()
                    .zip(nh.iter())
                    .all(|(constrain, tile)| constrain.satisfies(*tile));

                if fits {
                    return Some((
                        blueprint.tile_at(pos),
                        PyxelTile(*id, tileset.pyxel_file),
                        Transform {
                            translation: Vec3::new(
                                pos.0 as f32 * tileset.tile_width as f32,
                                -pos.1 as f32 * tileset.tile_height as f32,
                                0.,
                            ),
                            rotation: (*rotation).into(),
                            scale: Vec3::new(if *flipped { -1. } else { 1. }, 1., 1.),
                        },
                    ));
                }
            }
        }
    }
    None
}

pub fn create(blueprint: &RoomBlueprint, commands: &mut Commands, tileset: &Tileset) {
    for pos in blueprint.positions().into_iter() {
        if let Some((tile, pyxel_tile, transform)) = get_tile_components(pos, &blueprint, tileset) {
            let needs_collider = match tile {
                Tile::Wall | Tile::Door(_) => true,
                _ => false,
            };

            let mut ebuider = commands.spawn_bundle((tile, pyxel_tile, transform));

            if needs_collider {
                ebuider
                    .insert(Collider::AABB(Vec2::new(
                        tileset.tile_width as f32 / 2.0,
                        tileset.tile_height as f32 / 2.0,
                    )))
                    .insert(RigidBody {
                        velocity: Vec2::ZERO,
                    });
            }
        }
    }
}
