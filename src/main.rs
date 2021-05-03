#[macro_use]
mod common;
mod pyxel_plugin;
mod unreachable;

#[macro_use]
use bevy::prelude::*;
use bevy::render::camera::OrthographicProjection;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use heron::prelude::*;

use pyxel_plugin::PyxelPlugin;
use unreachable::UnreachableGame;

pub fn main() {
    App::build()
        .insert_resource(bevy::log::LogSettings {
            level: bevy::log::Level::DEBUG,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(PyxelPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(UnreachableGame)
        //--
        .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin::default())
        .add_plugin(bevy::wgpu::diagnostic::WgpuResourceDiagnosticsPlugin::default())
        .add_plugin(bevy::asset::diagnostic::AssetCountDiagnosticsPlugin::<
            Texture,
        >::default())
        .add_plugin(bevy::asset::diagnostic::AssetCountDiagnosticsPlugin::<
            ColorMaterial,
        >::default())
        //--
        .add_startup_system(on_startup.system())
        .add_startup_system(common::known_fonts::load_known_fonts.system())
        .add_system(debug.system())
        .run();
}

fn on_startup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle {
        orthographic_projection: OrthographicProjection {
            scale: 1./4.,
            ..Default::default()
        },
        ..OrthographicCameraBundle::new_2d()
    });
    info!("Started!");
}

fn debug(
    mut egui_context: ResMut<EguiContext>,
    query: Query<Entity>,
    mut transforms: Query<&mut Transform>,
    sprites: Query<&Sprite>,
    materials: Query<&Handle<ColorMaterial>>,
    pyxel_sprites: Query<&pyxel_plugin::PyxelSprite>,
) {
    egui::Window::new("Entities").show(egui_context.ctx(), |ui| {
        for e in query.iter() {
            ui.collapsing(format!("- {:?}", e), |ui| {
                if let Ok(mut t) = transforms.get_mut(e) {
                    ui.horizontal(|ui| {
                        ui.label(format!("Pos"));
                        ui.add(egui::widgets::DragValue::new(&mut t.translation.x).speed(0.1));
                        ui.add(egui::widgets::DragValue::new(&mut t.translation.y).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("Rot"));
                        let (_axis, mut angle) = t.rotation.to_axis_angle();
                        ui.add(egui::widgets::DragValue::new(&mut angle).speed(0.01));
                        t.rotation = Quat::from_rotation_z(angle);
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("Scale"));
                        ui.add(egui::widgets::DragValue::new(&mut t.scale.x).speed(0.01));
                        ui.add(egui::widgets::DragValue::new(&mut t.scale.y).speed(0.01));
                    });
                }
                if let Ok(s) = sprites.get(e) {
                    ui.label(format!("{:#?}", s));
                }
                if let Ok(m) = materials.get(e) {
                    ui.label(format!("{:#?}", m));
                }
                if let Ok(s) = pyxel_sprites.get(e) {
                    ui.label(format!("{:#?}", s));
                }
            });
        }
    });
}
