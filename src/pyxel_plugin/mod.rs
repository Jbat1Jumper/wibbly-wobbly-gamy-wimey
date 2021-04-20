use crate::common::*;
use bevy::prelude::*;
use core::ops::Deref;
use pyxel::Pyxel;
use resources::*;
use sprite_sheet::*;

mod resources;
mod sprite_sheet;

pub struct PyxelPlugin {
    text_resources: TextResources,
    sprite_resources: SpriteResources,
    frames: usize,
}

impl Plugin for PyxelPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder
            .add_system(create_sprites_for_sprites.system())
            .add_system(animate_sprites.system())
            .add_system(create_sprites_for_tiles.system())
            .insert_resource(PyxelResources::new(map! {
                "base.pyxel" => {
                    //pyxel::open("resources/base.pyxel")
                    pyxel::load_from_memory(
                        include_bytes!("resources/base.pyxel")
                    )
                    .expect("Problems loading base.pyxel")
                }
            }))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct PyxelTile(pub usize, pub &'static str);

#[derive(Clone, Debug)]
pub struct PyxelSprite {
    pub pyxel_file: &'static str,
    pub current_animation: AnimationId,
    // TODO: Use looping bevy::prelude::Timer instead of this
    pub current_animation_time: f64,
}


fn create_sprites_for_sprites(
    commands: Commands,
    query: Query<(&PyxelSprite, &Transform), (Without<Sprite>)>,
    mut pyxel: ResMut<PyxelResources>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (sprite, transform) in query.iter() {
        let PyxelSprite {
            pyxel_file,
            current_animation,
            current_animation_time,
        } = sprite;

        let material = pyxel.get_sprite_frame_material(
            current_animation,
            *current_animation_time,
            layer,
            pyxel_file,
        );

        commands.spawn_bundle(SpriteBundle {
            tranform: transform.clone(),
            material,
            ..Default::default()
        });
    }
}

fn animate_sprites(
    query: Query<(&mut PyxelSprite, &mut Handle<ColorMaterial>)>,
    mut time: Res<Time>,
    mut pyxel: ResMut<PyxelResources>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let delta = time.delta_seconds_f64();

    for (sprite, material) in query.iter() {
        let file = pyxel_files.get(&sprite.pyxel_file).unwrap();
        let duration = sprite_sheet.get_animation_duration(&sprite.current_animation).unwrap();
        sprite.current_animation_time = (sprite.current_animation_time + delta) % duration;

        material = pyxel.get_sprite_frame_material(
            current_animation,
            *current_animation_time,
            layer,
            pyxel_file,
        );
    }
}

fn create_sprites_for_tiles(
    commands: Commands,
    query: Query<(&TileRef, &Transform), (Without<Sprite>)>,
    mut pyxel: ResMut<PyxelResources>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (tile, transform) in query.iter() {
        let material = pyxel.get_tile_material(tile, &mut textures, &mut materials);
        commands.spawn_bundle(SpriteBundle {
            tranform: transform.clone(),
            material,
            ..Default::default()
        });
    }
}
