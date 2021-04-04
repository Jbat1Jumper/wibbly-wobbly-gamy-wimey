use super::room_gen::model::{RoomBlueprint, Tile};
use crate::common::*;
use glam::f32::*;
use legion::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileGraphic {
    Empty,
    Ground,
    Wall,
    Door,
}

pub struct Tileset {
    pub pyxel_file: &'static str,
    pub tile_constrains: HashMap<usize, [Constrain<TileGraphic>; 9]>,
    pub animations: Vec<AnimatedTile>,
    pub tile_width: usize,
    pub tile_height: usize,
}

impl Tileset {
    fn reference(&self) -> TilesetRef {
        TilesetRef {
            pyxel_file: self.pyxel_file,
        }
    }
}

pub struct AnimatedTile {
    pub name: &'static str,
    /// Indicates that any tile that matches a frame should be animated
    pub intrinsic: bool,
    pub frames: Vec<usize>,
}

impl From<Tile> for TileGraphic {
    fn from(t: Tile) -> Self {
        match t {
            Tile::Empty => TileGraphic::Empty,
            Tile::Ground => TileGraphic::Ground,
            Tile::Wall => TileGraphic::Wall,
            Tile::Door(_) => TileGraphic::Door,
        }
    }
}

fn get_tile_components(
    pos: (i32, i32),
    blueprint: &RoomBlueprint,
    tileset: &Tileset,
) -> Option<(TileRef, SpriteTransform, Position, Tile)> {
    let nh = blueprint.tile_neighborhood(pos);

    for flipped in &[false, true] {
        for rotation in &Rotation::all() {
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
                    .all(|(constrain, tile)| constrain.satisfies(&(*tile).into()));

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
                        blueprint.tile_at(pos),
                    ));
                }
            }
        }
    }
    None
}

fn create_tile_colliders(world: &mut World, tileset: &Tileset) {
    use crate::physics::*;
    let mut query = <(Entity, &Tile)>::query().filter(!component::<RigidBody2D>());

    let rigidbodies: Vec<_> = query
        .iter(world)
        .filter_map(|(e, tile)| match tile {
            Tile::Wall | Tile::Door(_) => Some((
                e.clone(),
                RigidBody2D::new(
                    Shape::AABB(
                        tileset.tile_width as f32 / 2.0,
                        tileset.tile_height as f32 / 2.0,
                    ),
                    true,
                ),
            )),
            _ => None,
        })
        .collect();
    for (e, rb) in rigidbodies {
        if let Some(mut entry) = world.entry(e) {
            entry.add_component(rb);
        }
    }
}

pub fn create(blueprint: &RoomBlueprint, world: &mut World, resources: &mut Resources) {
    let tileset = resources.get().expect("No tileset found");
    world.extend(
        blueprint
            .positions()
            .iter()
            .filter_map(|pos| get_tile_components(*pos, &blueprint, &tileset)),
    );
    create_tile_colliders(world, &tileset);
}
