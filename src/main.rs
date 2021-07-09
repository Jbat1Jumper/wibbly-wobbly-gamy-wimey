#[macro_use]
mod common;
mod plain_simple_physics;
mod playdate;
mod pyxel_plugin;
mod sprint;
mod unreachable;

#[macro_use]
use bevy::prelude::*;
use bevy::render::camera::OrthographicProjection;
use bevy_egui::EguiPlugin;

use pyxel_plugin::PyxelPlugin;
use unreachable::UnreachableGame;

pub fn main() {
    App::build()
        .insert_resource(bevy::log::LogSettings {
            level: bevy::log::Level::DEBUG,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(plain_simple_physics::PlainSimplePhysicsPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(PyxelPlugin)
        //.add_plugin(UnreachableGame)
        .add_system(root_ui::root_editor_ui.system())
        .add_plugin(playdate::PlaydateModelsPlugin)
        .add_plugin(sprint::SprintGame)
        //.add_startup_system(setup_orhographic_camera.system())
        //.add_startup_system(common::known_fonts::load_known_fonts.system())
        .run();
}

fn setup_orhographic_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle {
        orthographic_projection: OrthographicProjection {
            scale: 1. / 4.,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(0., 0., 1000.),
            scale: Vec3::new(1., 1., 1.),
            ..Default::default()
        },
        ..OrthographicCameraBundle::new_2d()
    });
    info!("Started!");
}


/// TODO: Make plugin
mod root_ui {
    use bevy::{app::AppExit, prelude::*};
    use bevy_egui::{egui, EguiContext};
    pub fn root_editor_ui(
        mut commands: Commands,
        egui_context: ResMut<EguiContext>,
        mut exit: EventWriter<AppExit>,
    ) {
        egui::TopPanel::top("root_editor_ui").show(egui_context.ctx(), |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        exit.send(AppExit);
                    }
                });
            });
        });
    }
}
