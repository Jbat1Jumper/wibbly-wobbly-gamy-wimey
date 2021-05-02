use super::UnScene;
use crate::common::*;
use bevy::prelude::*;
use std::time::Duration;

pub struct Intro;

impl Plugin for Intro {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(SystemSet::on_enter(UnScene::Intro).with_system(enter.system()))
            .add_system_set(SystemSet::on_update(UnScene::Intro).with_system(update.system()))
            .add_system_set(SystemSet::on_exit(UnScene::Intro).with_system(exit.system()))
            .insert_resource(IntroState {
                remaining_time: Duration::from_secs(1),
            });
    }
}

struct IntroState {
    remaining_time: Duration,
}

fn enter(mut commands: Commands, kf: Res<KnownFonts>) {
    commands
        .spawn()
        .insert_bundle(kf.create_text("SOGA", (20., 20.), KnownFont::LiberationMono, 20.))
        .insert(IntroState {
            remaining_time: Duration::from_secs(1),
        });
}

fn update(time: Res<Time>, mut intro: ResMut<IntroState>, mut scene: ResMut<State<UnScene>>) {
    if intro.remaining_time > time.delta() {
        intro.remaining_time -= time.delta();
    } else {
        scene.set(UnScene::MainMenu).unwrap();
    }
}

fn exit(mut commands: Commands, to_remove: Query<Entity, With<Text>>) {
    for e in to_remove.iter() {
        commands.entity(e).despawn();
    }
}
