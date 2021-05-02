use bevy::prelude::*;
use crate::common::*;
use super::UnScene;

pub struct MainMenu;

impl Plugin for MainMenu {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(SystemSet::on_enter(UnScene::MainMenu).with_system(enter.system()))
            .add_system_set(SystemSet::on_update(UnScene::MainMenu).with_system(update.system()))
            .add_system_set(SystemSet::on_exit(UnScene::MainMenu).with_system(exit.system()));
    }
}

fn enter(mut commands: Commands, kf: Res<KnownFonts>) {
    let font = KnownFont::LiberationMono;
    
    commands.spawn_batch(vec![
        kf.create_text("Lost and Found", (10., 10.), font, 12.),
        kf.create_text("Press space to start!", (10., 25.), font, 18.),
        kf.create_text("Or press esc to exit", (10., 50.), font, 12.),
    ]);
}

fn update(keyboard: Res<Input<KeyCode>>, mut scene: ResMut<State<UnScene>>) {
    if keyboard.just_pressed(KeyCode::J) | keyboard.just_pressed(KeyCode::Space) {
        scene.set(UnScene::Game);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        scene.set(UnScene::Exit);
    }
}

fn exit(mut commands: Commands, to_remove: Query<Entity, With<Text>>) {
    for e in to_remove.iter() {
        commands.entity(e).despawn();
    }
}
