use super::room_gen::model::{RoomBlueprint, Tile};
use crate::common::*;
use crate::pyxel_plugin::PyxelTile;
use bevy::prelude::*;
use heron::prelude::*;
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
                let constrains = constrains.clone().rotate(rotation);
                let constrains = if *flipped {
                    constrains.clone().flip_horizontally()
                } else {
                    constrains.clone()
                };
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
                                pos.1 as f32 * tileset.tile_height as f32,
                                0.,
                            ),
                            rotation: (*rotation).into(),
                            scale: Vec3::new(1., if *flipped { 1. } else { -1. }, 1.),
                        },
                    ));
                }
            }
        }
    }
    None
}

pub fn create(blueprint: &RoomBlueprint, cmd: &mut Commands, tileset: &Tileset) {
    let tile_components = blueprint
        .positions()
        .into_iter()
        .filter_map(|pos| get_tile_components(pos, &blueprint, tileset));

    let (need_collider, without_collider): (Vec<_>, Vec<_>) =
        tile_components.partition(|c| match c.0 {
            Tile::Wall | Tile::Door(_) => true,
            _ => false,
        });

    let with_collider: Vec<_> = need_collider
        .into_iter()
        .map(|(tile, pyxel_tile, transform)| {
            (
                tile,
                pyxel_tile,
                transform,
                Body::Cuboid {
                    half_extends: Vec3::new(
                        tileset.tile_width as f32 / 2.0,
                        tileset.tile_height as f32 / 2.0,
                        1.,
                    ),
                },
                BodyType::Static,
            )
        })
        .collect();

    cmd.spawn_batch(without_collider);
    cmd.spawn_batch(with_collider);
}
