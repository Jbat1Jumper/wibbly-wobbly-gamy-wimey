use crate::common::*;
use core::ops::Deref;
use ggez;
use ggez::event::{run as launch, EventHandler};
use ggez::graphics;
pub use ggez::GameResult;
use glam::f32::Vec2;
use legion::*;
use pyxel::Pyxel;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

trait Game {
    type SceneRef;
}

pub struct GgezBackend {
    ggez_ctx: ggez::Context,
    ggez_events_loop: ggez::event::EventsLoop,
    text_resources: TextResources,
    sprite_resources: SpriteResources,
    frames: usize,
}

impl PlugIn for GgezBackend {
    fn init(&mut self, world: &mut World, resources: &mut Resources) {}
    fn update(&mut self, world: &mut World, resources: &mut Resources) {
        let mut button_events: Vec<(Button, ButtonState)> = vec![];
        self.frames += 1;
        let GgezBackend {
            ggez_ctx,
            ggez_events_loop,
            ..
        } = self;
        ggez_ctx.timer_context.tick();
        ggez_events_loop.poll_events(|event| {
            ggez_ctx.process_event(&event);
            match event {
                ggez::event::winit_event::Event::WindowEvent { event, .. } => match event {
                    ggez::event::winit_event::WindowEvent::CloseRequested => {
                        ggez::event::quit(ggez_ctx)
                    }
                    ggez::event::winit_event::WindowEvent::KeyboardInput {
                        input:
                            ggez::event::winit_event::KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    } => match Self::key_binding(keycode) {
                        Some(button) => button_events.push((button, Self::state_binding(state))),
                        None => (),
                    },
                    // `CloseRequested` and `KeyboardInput` events won't appear here.
                    x => (), //println!("Other window event fired: {:?}", x),
                },

                x => (), //println!("Device event fired: {:?}", x),
            }
        });

        let MustQuit(mut must_quit) = *resources.get_mut_or_default();
        must_quit = must_quit || !self.ggez_ctx.continuing;
        resources.insert(MustQuit(must_quit));

        resources.insert(button_events);
        let cmds: Vec<SceneCommand> = vec![];
        resources.insert(cmds);
        resources.insert(LastFrameDuration(ggez::timer::delta(&self.ggez_ctx)));
        resources.insert(CurrentFPS(ggez::timer::fps(&self.ggez_ctx)));
        resources.insert(CurrentFrame(self.frames));
    }

    fn draw(&mut self, world: &World, resources: &Resources) {
        graphics::clear(&mut self.ggez_ctx, [0.1, 0.2, 0.3, 1.0].into());

        let mut query = <(&Position, &Text)>::query();
        for (position, text) in query.iter(world) {
            let Position(pos) = position;
            let rtext = self
                .text_resources
                .render_text(&mut self.ggez_ctx, text)
                .expect("Error drawing text");

            graphics::draw(&mut self.ggez_ctx, rtext, (pos.clone(),));
        }

        // TODO: The ordering and layers in this part are hardcoded
        {
            let pyxel_files = resources.get_mut().expect("No pyxel files");

            let mut query = <(&TileRef, &SpriteTransform, &Position)>::query();
            for (tile, transform, pos) in query.iter(world) {
                self.draw_tile(tile, transform, &*pyxel_files, pos)
                    .expect("Error drawing tile");
            }

            let mut query = <(&Sprite, &SpriteTransform, &Position)>::query();
            for (sprite, transform, pos) in query.iter(world) {
                self.draw_sprite(sprite, &"shadows".into(), transform, &*pyxel_files, pos);
            }
            for (sprite, transform, pos) in query.iter(world) {
                self.draw_sprite(sprite, &"main".into(), transform, &*pyxel_files, pos);
            }
        }

        let CurrentFrame(frame) = *resources.get().expect("Error reading current frame");
        if (frame % 100) == 0 {
            let CurrentFPS(fps) = *resources.get().expect("Error reading fps");
            println!("FPS: {}", fps);
        }

        graphics::present(&mut self.ggez_ctx);
    }
}

pub trait PlugIn {
    fn init(&mut self, world: &mut World, resources: &mut Resources);

    fn update(&mut self, world: &mut World, resources: &mut Resources);

    fn draw(&mut self, world: &World, resources: &Resources);
}

pub struct MustQuit(pub bool);

impl Default for MustQuit {
    fn default() -> Self {
        MustQuit(false)
    }
}

pub struct LastFrameDuration(pub Duration);

pub struct PyxelFiles(pub HashMap<&'static str, pyxel::Pyxel>);

pub struct CurrentFPS(pub f64);

pub struct CurrentFrame(pub usize);

impl GgezBackend {
    pub fn new(
        (ggez_ctx, ggez_events_loop): (ggez::Context, ggez::event::EventsLoop),
        sprite_resources: SpriteResources,
    ) -> GameResult<GgezBackend> {
        Ok(GgezBackend {
            ggez_ctx,
            ggez_events_loop,
            sprite_resources,
            frames: 0,
            text_resources: TextResources {
                loaded_fonts: HashMap::new(),
                rendered_texts: HashMap::new(),
            },
        })
    }

    fn key_binding(key: ggez::input::keyboard::KeyCode) -> Option<Button> {
        use ggez::input::keyboard::KeyCode;
        match key {
            KeyCode::Escape => Some(Button::Start),
            KeyCode::W => Some(Button::Up),
            KeyCode::A => Some(Button::Left),
            KeyCode::S => Some(Button::Down),
            KeyCode::D => Some(Button::Right),
            KeyCode::J => Some(Button::A),
            KeyCode::K => Some(Button::B),
            _ => None,
        }
    }

    fn state_binding(state: ggez::event::winit_event::ElementState) -> ButtonState {
        match state {
            ggez::event::winit_event::ElementState::Pressed => ButtonState::Pressed,
            ggez::event::winit_event::ElementState::Released => ButtonState::Released,
        }
    }

    fn draw_sprite(
        &mut self,
        sprite: &Sprite,
        layer: &LayerId,
        tranform: &SpriteTransform,
        pyxel_files: &PyxelFiles,
        position: &Position,
    ) -> GameResult {
        let Sprite {
            pyxel_file,
            current_animation,
            current_animation_time,
        } = sprite;
        let img = self.sprite_resources.get_pyxel_frame(
            &mut self.ggez_ctx,
            current_animation,
            *current_animation_time,
            layer,
            pyxel_files,
            pyxel_file,
        )?;
        let SpriteTransform { rotation, flipped } = tranform;
        let Position(pos) = position;
        ggez::graphics::draw(
            &mut self.ggez_ctx,
            img,
            ggez::graphics::DrawParam::default()
                .dest(*pos * 4.0)
                .offset([0.5, 0.5])
                .rotation(rotation.radians())
                .scale([4.0, 4.0]),
        )?;

        Ok(())
    }

    fn draw_tile(
        &mut self,
        tile: &TileRef,
        tranform: &SpriteTransform,
        pyxel_files: &PyxelFiles,
        position: &Position,
    ) -> GameResult {
        let TileRef(index, TilesetRef { pyxel_file }) = tile;
        let img = self.sprite_resources.get_pyxel_tile(
            &mut self.ggez_ctx,
            *index,
            pyxel_files,
            pyxel_file,
        )?;

        let SpriteTransform { rotation, flipped } = tranform;
        let Position(pos) = position;

        ggez::graphics::draw(
            &mut self.ggez_ctx,
            img,
            ggez::graphics::DrawParam::default()
                .dest(*pos * 4.0)
                .offset([0.5, 0.5])
                .rotation(rotation.radians())
                .scale([4.0, 4.0]),
        )?;

        //panic!("No draw sprite me");
        Ok(())
    }
}

pub struct SpriteResources {
    pub loaded_tiles: HashMap<(&'static str, usize), ggez::graphics::Image>,
    pub loaded_frames: HashMap<(LayerId, FrameId, &'static str), ggez::graphics::Image>,
}

impl Default for SpriteResources {
    fn default() -> SpriteResources {
        SpriteResources {
            loaded_tiles: HashMap::new(),
            loaded_frames: HashMap::new(),
        }
    }
}

impl SpriteResources {
    fn get_pyxel_frame<'a>(
        &'a mut self,
        ctx: &mut ggez::Context,
        current_animation: &AnimationId,
        current_animation_time: f64,
        layer: &LayerId,
        pyxel_files: &PyxelFiles,
        pyxel_file: &'static str,
    ) -> GameResult<&'a ggez::graphics::Image> {
        let file = pyxel_files
            .0
            .get(pyxel_file)
            .ok_or(ggez::error::GameError::ResourceLoadError(pyxel_file.into()))?;

        let frame = file
            .get_frame_at(current_animation, current_animation_time)
            .map_err(ggez::error::GameError::RenderError)?;

        if !self
            .loaded_frames
            .contains_key(&(layer.clone(), frame, pyxel_file))
        {
            let data = file
                .get_frame_data_in_rgba8(&frame, layer)
                .map_err(ggez::error::GameError::ResourceLoadError)?;
            let mut img = ggez::graphics::Image::from_rgba8(
                ctx,
                file.canvas().tile_width(),
                file.canvas().tile_height(),
                &data,
            )?;
            img.set_filter(ggez::graphics::FilterMode::Nearest);

            self.loaded_frames
                .insert((layer.clone(), frame, pyxel_file), img);
        }

        Ok(self
            .loaded_frames
            .get(&(layer.clone(), frame, pyxel_file))
            .unwrap())
    }

    fn get_pyxel_tile<'a>(
        &'a mut self,
        ctx: &mut ggez::Context,
        index: usize,
        pyxel_files: &PyxelFiles,
        pyxel_file: &'static str,
    ) -> GameResult<&'a ggez::graphics::Image> {
        if !self.loaded_tiles.contains_key(&(pyxel_file, index)) {
            if let Some(file) = pyxel_files.0.get(pyxel_file) {
                let data: Vec<u8> = file.tileset().images()[index]
                    .to_rgba()
                    .pixels()
                    .map(|pyxel| &pyxel.0)
                    .flatten()
                    .cloned()
                    .collect();
                let mut img = ggez::graphics::Image::from_rgba8(
                    ctx,
                    file.tileset().tile_width(),
                    file.tileset().tile_height(),
                    &data,
                )?;
                img.set_filter(ggez::graphics::FilterMode::Nearest);

                self.loaded_tiles.insert((pyxel_file, index), img);
            } else {
                return Err(ggez::error::GameError::ResourceLoadError(pyxel_file.into()));
            }
        }
        Ok(self.loaded_tiles.get(&(pyxel_file, index)).unwrap())
    }
}

pub struct TextResources {
    rendered_texts: HashMap<Text, graphics::Text>,
    loaded_fonts: HashMap<Font, graphics::Font>,
}

impl TextResources {
    pub fn get_font<'a>(
        &'a mut self,
        ctx: &mut ggez::Context,
        font: &Font,
    ) -> GameResult<&'a graphics::Font> {
        if !self.loaded_fonts.contains_key(font) {
            let gfont = graphics::Font::new_glyph_font_bytes(ctx, font.truetype_font_bytes())?;
            self.loaded_fonts.insert(*font, gfont);
        }
        Ok(self.loaded_fonts.get(font).unwrap())
    }

    pub fn render_text<'a>(
        &'a mut self,
        ctx: &mut ggez::Context,
        text: &Text,
    ) -> GameResult<&'a graphics::Text> {
        if !self.rendered_texts.contains_key(text) {
            let Text { string, font, size } = text;
            let gfont = self.get_font(ctx, font)?;
            let rtext = graphics::Text::new((string.clone(), *gfont, *size as f32));
            self.rendered_texts.insert(text.clone(), rtext);
        }
        Ok(self.rendered_texts.get(text).unwrap())
    }
}
