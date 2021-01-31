use crate::common::*;
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


pub struct Backend
{
    ggez_ctx: ggez::Context,
    ggez_events_loop: ggez::event::EventsLoop,
    text_resources: TextResources,
    frames: usize,
}

impl Backend {
    pub fn new(
        (ggez_ctx, ggez_events_loop): (ggez::Context, ggez::event::EventsLoop),
    ) -> GameResult<Backend> {
        Ok(Backend {
            ggez_ctx,
            ggez_events_loop,
            frames: 0,
            text_resources: TextResources {
                loaded_fonts: HashMap::new(),
                rendered_texts: HashMap::new(),
            },
        })
    }

    pub fn continuing(&self) -> bool {
        self.ggez_ctx.continuing
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

    pub fn draw_text(&mut self, text: &Text, position: &Position) -> GameResult {
        let Position(pos) = position;
        let rtext = self.text_resources.render_text(&mut self.ggez_ctx, text)?;
        graphics::draw(&mut self.ggez_ctx, rtext, (pos.clone(),))
    }

    pub fn delta_time(&mut self) -> Duration {
        ggez::timer::delta(&self.ggez_ctx)
    }

    pub fn quit(&mut self) -> GameResult {
        ggez::event::quit(&mut self.ggez_ctx);
        Ok(())
    }
    pub fn clear(&mut self) -> GameResult {
        graphics::clear(&mut self.ggez_ctx, [0.1, 0.2, 0.3, 1.0].into());
        Ok(())
    }

    pub fn get_fps(&mut self) -> f64 {
        ggez::timer::fps(&self.ggez_ctx)
    }

    pub fn present(&mut self) -> GameResult {
        graphics::present(&mut self.ggez_ctx)
    }

    pub fn current_frame(&self) -> usize {
        self.frames
    }

    pub fn draw_sprite(&mut self, sprite: &Sprite, tranform: &SpriteTransform, position: &Position) -> GameResult {
        panic!("No draw sprite me");
    }

    pub fn poll_events(&mut self) -> Vec<(Button, ButtonState)> {
        let mut button_events: Vec<(Button, ButtonState)> = vec![];
        self.frames += 1;
        let Backend {
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
        button_events
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
            let gfont = graphics::Font::new(ctx, font.resource_path())?;
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
