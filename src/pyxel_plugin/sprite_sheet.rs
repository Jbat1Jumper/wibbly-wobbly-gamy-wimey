use pyxel::Pyxel;

pub type AnimationId = String;
pub type FrameId = u32;
pub type LayerId = String;

pub trait SpriteSheet {
    fn get_size(&self) -> (usize, usize);
    fn get_animations(&self) -> Vec<AnimationId>;
    fn get_layers(&self) -> Vec<LayerId>;
    fn get_animation_frames(&self, animation: &AnimationId) -> Result<Vec<(FrameId, f64)>, String>;

    fn get_animation_duration(&self, animation: &AnimationId) -> Result<f64, String> {
        Ok(self
            .get_animation_frames(animation)?
            .iter()
            .map(|(_, d)| d)
            .sum())
    }

    fn get_frame_data_in_rgba8(&self, frame: &FrameId, layer: &LayerId) -> Result<Vec<u8>, String>;

    fn get_frame_at(&self, animation: &AnimationId, at_time: f64) -> Result<FrameId, String> {
        let mut t = 0.0;
        for (frame, duration) in self.get_animation_frames(animation)?.iter() {
            if t + duration > at_time {
                return Ok(*frame);
            }
            t += duration;
        }
        Err(format!(
            "Time {} out of bounds in animation {}",
            at_time, animation
        ))
    }
}

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

    fn get_animation_frames(&self, animation: &AnimationId) -> Result<Vec<(FrameId, f64)>, String> {
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
        let (x, y) = (frame % width, frame / width);

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
            .map(|p| Vec::from(p.2 .0))
            .flatten()
            .collect())
    }
}


