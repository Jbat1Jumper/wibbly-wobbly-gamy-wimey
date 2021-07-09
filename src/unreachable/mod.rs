use bevy::prelude::*;
use crate::root_ui::*;

mod scenes;

pub struct UnreachableGame;

impl Plugin for UnreachableGame {
    fn build(&self, app: &mut AppBuilder) {
        app
            // .see("https://github.com/bevyengine/bevy/issues/69")
            // .require(RootUiPlugin)
            // .should_be_implemented()
            .add_plugin(scenes::LoadGameScenes)
            .add_system(debug::debug_window.system());
    }
}

mod debug {
    use crate::{common, plain_simple_physics, pyxel_plugin};
    use bevy::prelude::*;
    use crate::root_ui::*;

    pub fn debug_window(
        egui_context: ResMut<EguiContext>,
        query: Query<Entity>,
        mut transforms: Query<&mut Transform>,
        global_transforms: Query<&GlobalTransform>,
        sprites: Query<&Sprite>,
        materials: Query<&Handle<ColorMaterial>>,
        pyxel_sprites: Query<&pyxel_plugin::PyxelSprite>,
        names: Query<&Name>,
        vehicles: Query<&common::Vehicle>,
        colliders: Query<&plain_simple_physics::Collider>,
        rigid_bodies: Query<&plain_simple_physics::RigidBody>,
        move_transforms: Query<&plain_simple_physics::MoveTransform>,
        contacts: Res<plain_simple_physics::CurrentContacts>,
    ) {
        egui::Window::new("CurrentContacts").show(egui_context.ctx(), |ui| {
            ui.label(format!("{:#?}", *contacts));
        });
        egui::Window::new("Entities").show(egui_context.ctx(), |ui| {
            for e in query.iter() {
                let name: String = if let Ok(n) = names.get(e) {
                    n.as_str().into()
                } else {
                    format!("{:?}", e)
                };
                ui.collapsing(name, |ui| {
                    if let Ok(mut t) = transforms.get_mut(e) {
                        ui.horizontal(|ui| {
                            ui.label(format!("Pos"));
                            ui.add(egui::widgets::DragValue::new(&mut t.translation.x).speed(0.1));
                            ui.add(egui::widgets::DragValue::new(&mut t.translation.y).speed(0.1));
                            ui.add(egui::widgets::DragValue::new(&mut t.translation.z).speed(0.1));
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
                    if let Ok(mt) = move_transforms.get(e) {
                        ui.label(format!("{:#?}", mt));
                    }
                    if let Ok(c) = colliders.get(e) {
                        ui.label(format!("{:#?}", c));
                    }
                    if let Ok(rb) = rigid_bodies.get(e) {
                        ui.label(format!("{:#?}", rb));
                    }
                    if let Ok(t) = global_transforms.get(e) {
                        ui.label(format!("{:#?}", t));
                    }
                    if let Ok(v) = vehicles.get(e) {
                        ui.label(format!("{:#?}", v));
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
}
