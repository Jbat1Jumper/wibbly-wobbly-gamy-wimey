use bevy::prelude::*;
use crate::common::*;
use glam::f32::Vec2;
use std::time::Duration;
use super::UnScene;

pub struct Intro;

impl Plugin for Intro {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(SystemSet::on_enter(UnScene::Intro).with_system(enter.system()))
            .add_system_set(SystemSet::on_update(UnScene::Intro).with_system(update.system()))
            .add_system_set(SystemSet::on_exit(UnScene::Intro).with_system(exit.system()))
    }
}

struct IntroState {
    remaining_time: Duration,
}

fn enter(mut commands: Commands) {
    let font = Font::LiberationMono;
    commands
        .spawn()
        .insert_bundle((Text::new("SOGA", font, 12), Position(Vec2::new(20.0, 20.0))))
        .insert_resource(IntroState {
            remaining_time: Duration::from_secs(1),
        });
}

fn update(
    lfd: Res<LastFrameDuration>,
    mut intro: ResMut<IntroState>,
    mut scene: ResMut<Sate<UnScene>>,
) {
    let LastFrameDuration(delta) = lfd;
    if *intro.remaining_time > *delta {
        *intro.remaining_time -= *delta;
    } else {
        scene.set(UnScene::MainMenu).unwrap();
    }
}

fn exit(mut commands: Commands, to_remove: Query<Entity, With<Text>>) {
    for e in to_remove.iter() {
        commands.entity(e).despawn();
    }
}
