use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum KnownFont {
    LiberationMono,
}

pub struct KnownFonts {
    fonts: HashMap<KnownFont, Handle<Font>>,
}

impl KnownFonts {
    pub fn get(&self, font: KnownFont) -> Handle<Font> {
        self.fonts.get(&font).expect("Font not loaded").clone()
    }

pub fn create_text<S: Into<String>, P: Into<Vec2>>(&self, text: S, pos: P, font: KnownFont, size: f32) -> Text2dBundle {
    Text2dBundle {
        transform: Transform {
            translation: pos.into().extend(0.),
            ..Default::default()
        },
        text: Text::with_section(
            text.into(),
            TextStyle {
                font: self.get(font),
                font_size: size,
                color: Color::ALICE_BLUE,
            },
            TextAlignment::default(),
        ),
        ..Default::default()
    }
}

}

pub fn load_known_fonts(mut commands: Commands, mut font_assets: ResMut<Assets<Font>>) {
    let lm_bytes: Vec<u8> = include_bytes!("resources/LiberationMono-Regular.ttf").into_iter().cloned().collect();
    let handle =
        font_assets.add(Font::try_from_bytes(lm_bytes).expect("Couldnt load liberation mono"));

    commands.insert_resource(KnownFonts {
        fonts: {
            let mut m = HashMap::new();
            m.insert(KnownFont::LiberationMono, handle);
            m
        },
    });
}
