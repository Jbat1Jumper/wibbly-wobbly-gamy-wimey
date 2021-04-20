use bevy::prelude::*;
use crate::common::*;
use glam::f32::Vec2;
use super::UnScene;

pub struct MainMenu;

impl Plugin for MainMenu {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(SystemSet::on_enter(UnScene::MainMenu).with_system(enter.system()))
            .add_system_set(SystemSet::on_update(UnScene::MainMenu).with_system(update.system()))
            .add_system_set(SystemSet::on_exit(UnScene::MainMenu).with_system(exit.system()))
    }
}

pub fn enter(mut commands: Commands) { 
    let font = Font::LiberationMono;
    commands.spawn_batch(vec![
        (
            Text::new("Lost and Found", font, 12),
            Position(Vec2::new(10.0, 10.0)),
        ),
        (
            Text::new("Press space to start!", font, 18),
            Position(Vec2::new(10.0, 25.0)),
        ),
        (
            Text::new("Or press esc to exit", font, 12),
            Position(Vec2::new(10.0, 50.0)),
        ),
    ]);
}

fn update(keyboard: Res<Input<KeyCode>>, mut scene: ResMut<Sate<UnScene>>) {
    if keyboard.just_pressed(KeyCode::J) {
        scene.set(UnScene::GameScene);
    }
    if keyboard.just_pressed(KeyCode::Esc) {
        scene.set(UnScene::Exit);
    }
}

fn exit(mut commands: Commands, to_remove: Query<Entity, With<Text>>) {
    for e in to_remove.iter() {
        commands.entity(e).despawn();
    }
}
