use super::*;
use bevy::render::texture::*;
use std::collections::HashMap;

pub struct PyxelResources {
    pub pyxel_files: HashMap<&'static str, pyxel::Pyxel>,
    pub loaded_tiles: HashMap<PyxelTile, Handle<ColorMaterial>>,
    pub loaded_frames: HashMap<(LayerId, FrameId, &'static str), Handle<ColorMaterial>>,
}

impl PyxelResources {
    pub fn new(pyxel_files: HashMap<&'static str, pyxel::Pyxel>) -> Self {
        Self {
            pyxel_files,
            loaded_tiles: HashMap::new(),
            loaded_frames: HashMap::new(),
        }
    }

    pub fn get_sprite_frame_material(
        &mut self,
        current_animation: &AnimationId,
        current_animation_time: f64,
        layer: &LayerId,
        pyxel_file: &'static str,
        textures: &mut Assets<Texture>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        let file = self.pyxel_files.get(pyxel_file).unwrap();

        let frame = file
            .get_frame_at(current_animation, current_animation_time)
            .unwrap();

        if !self
            .loaded_frames
            .contains_key(&(layer.clone(), frame, pyxel_file))
        {
            let data = file
                .get_frame_data_in_rgba8(&frame, layer)
                .expect("Resource load error");
            let texture = Texture::new_fill(
                Extent3d::new(
                    file.tileset().tile_width() as u32,
                    file.tileset().tile_height() as u32,
                    0,
                ),
                TextureDimension::D2,
                &data,
                TextureFormat::Rgba8Uint,
            );

            let texture_handle = textures.add(texture);
            let material_handle = materials.add(texture_handle.into());

            self.loaded_frames
                .insert((layer.clone(), frame, pyxel_file), material_handle);
        }

        self.loaded_frames
            .get(&(layer.clone(), frame, pyxel_file))
            .unwrap()
            .clone()
    }

    pub fn get_tile_material(
        &mut self,
        tile: &PyxelTile,
        textures: &mut Assets<Texture>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        if !self.loaded_tiles.contains_key(tile) {
            let material = self._load_tile_material(tile, textures, materials);
            self.loaded_tiles.insert(*tile, material);
        }

        self.loaded_tiles.get(tile).unwrap().clone()
    }

    fn _load_tile_material(
        &mut self,
        tile: &PyxelTile,
        textures: &mut Assets<Texture>,
        materials: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        let file = self.pyxel_files.get(tile.1).unwrap();
        let data: Vec<u8> = file.tileset().images()[tile.0]
            .to_rgba()
            .pixels()
            .map(|p| &p.0)
            .flatten()
            .cloned()
            .collect();

        let texture = Texture::new_fill(
            Extent3d::new(
                file.tileset().tile_width() as u32,
                file.tileset().tile_height() as u32,
                0,
            ),
            TextureDimension::D2,
            &data,
            TextureFormat::Rgba8Uint,
        );

        let texture_handle = textures.add(texture);
        let material_handle = materials.add(texture_handle.into());
        material_handle
    }
}
