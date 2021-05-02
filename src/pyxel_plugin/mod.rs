use bevy::prelude::*;
use resources::*;
use sprite_sheet::*;

mod resources;
mod sprite_sheet;

pub struct PyxelPlugin;

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
                        include_bytes!("../unreachable/resources/base.pyxel")
                    )
                    .expect("Problems loading base.pyxel")
                }
            }));
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
    mut commands: Commands,
    query: Query<(Entity, &PyxelSprite, &Transform), Without<Sprite>>,
    mut pyxel: ResMut<PyxelResources>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, sprite, transform) in query.iter() {
        let PyxelSprite {
            pyxel_file,
            current_animation,
            current_animation_time,
        } = sprite;

        let material = pyxel.get_sprite_frame_material(
            current_animation,
            *current_animation_time,
            &"main".into(),
            pyxel_file,
            &mut textures,
            &mut materials,
        );


        commands.entity(entity).insert_bundle(SpriteBundle {
            transform: transform.clone(),
            material,
            ..Default::default()
        });
    }
}

fn animate_sprites(
    mut query: Query<(&mut PyxelSprite, &mut Handle<ColorMaterial>)>,
    time: Res<Time>,
    mut pyxel: ResMut<PyxelResources>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let delta = time.delta_seconds_f64();

    for (mut sprite, mut material) in query.iter_mut() {
        let file = pyxel.pyxel_files.get(&sprite.pyxel_file).unwrap();
        let duration = file.get_animation_duration(&sprite.current_animation).unwrap();
        sprite.current_animation_time = (sprite.current_animation_time + delta) % duration;

        *material = pyxel.get_sprite_frame_material(
            &sprite.current_animation,
            sprite.current_animation_time,
            &"main".into(),
            sprite.pyxel_file,
            &mut textures,
            &mut materials,
        );
    }
}

fn create_sprites_for_tiles(
    mut commands: Commands,
    query: Query<(Entity, &PyxelTile, &Transform), Without<Sprite>>,
    mut pyxel: ResMut<PyxelResources>,
    mut textures: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, tile, transform) in query.iter() {
        let material = pyxel.get_tile_material(tile, &mut textures, &mut materials);
        commands.entity(entity).insert_bundle(SpriteBundle {
            transform: transform.clone(),
            material,
            ..Default::default()
        });
    }
}
