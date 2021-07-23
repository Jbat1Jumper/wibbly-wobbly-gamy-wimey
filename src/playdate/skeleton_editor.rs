use crate::root_ui::*;
use bevy::prelude::*;
use super::SkeletonDatabase;
use super::skeleton;

/*

pub struct SkeletonEditor {
    skeleton_name: String,
}

impl SkeletonEditor {
    pub fn for_skeleton(skeleton_name: String) -> Self {
        Self { skeleton_name }
    }

    pub fn render_editors(
        mut commands: Commands,
        egui_context: ResMut<EguiContext>,
        mut editors: Query<(Entity, &mut SkeletonEditor)>,
        mut skeleton_db: ResMut<SkeletonDatabase>,
    ) {
        for (editor_eid, mut editor) in editors.iter_mut() {
            let mut open = true;
            egui::Window::new(format!("Skeleton Editor: {}", editor.skeleton_name))
                .open(&mut open)
                .show(egui_context.ctx(), |ui| {
                    let md = skeleton_db.skeletons.get_mut(&editor.skeleton_name).unwrap();

                    let mut changes = vec![];

                    let mut roots: Vec<_> = md
                        .entries
                        .iter()
                        .filter(|(_, entry)| entry.attached_to == None)
                        .collect();
                    roots.sort_by_key(|(eid, _)| *eid);
                    for (id, entry) in roots {
                        edit_part(ui, *id, &entry.part, &md, &mut changes);
                    }

                    egui::ComboBox::from_id_source(-1)
                        .selected_text("Add new part to root")
                        .show_ui(ui, |ui| {
                            if ui.button("Translation").clicked() {
                                changes.push(skeleton::Change::AddToRoot(skeleton::Part::Translation(
                                    0., 0., 0.,
                                )));
                            }
                            if ui.button("Rotation").clicked() {
                                changes.push(skeleton::Change::AddToRoot(skeleton::Part::Rotation(
                                    skeleton::FloatValue::Constant(0.),
                                )));
                            }
                            if ui.button("EndEffector").clicked() {
                                changes.push(skeleton::Change::AddToRoot(skeleton::Part::EndEffector(
                                    "EFFECTOR NAME".into(),
                                )));
                            }
                        });

                    for change in changes {
                        md.apply(change).expect("Invalid part id");
                    }
                });
            if !open {
                commands.entity(editor_eid).despawn_recursive();
            }
        }
    }
}

fn part_name(part: &skeleton::Part) -> String {
    match part {
        skeleton::Part::Translation(x, y, z) => format!("Translation  ({}, {}, {})", x, y, z),
        skeleton::Part::Rotation(value) => match value {
            skeleton::FloatValue::Constant(c) => format!("Rotation by {} deg", c),
            skeleton::FloatValue::Variable(name) => format!("Rotation by {} deg", name),
        },
        skeleton::Part::EndEffector(name) => format!("EndEffector {}", name),
    }
}

fn edit_part(
    ui: &mut egui::Ui,
    id: usize,
    part: &skeleton::Part,
    md: &skeleton::Definition,
    changes: &mut Vec<skeleton::Change>,
) {
    ui.horizontal(|ui| {
        ui.label("▶");
        match part {
            skeleton::Part::Translation(x, y, z) => {
                let mut p = (*x, *y, *z);
                ui.label("Translate by (");
                ui.add(egui::DragValue::new(&mut p.0).speed(0.1));
                ui.add(egui::DragValue::new(&mut p.1).speed(0.1));
                ui.add(egui::DragValue::new(&mut p.2).speed(0.1));
                ui.label(")");

                if p != (*x, *y, *z) {
                    changes.push(skeleton::Change::Replace(
                        id,
                        skeleton::Part::Translation(p.0, p.1, p.2),
                    ));
                }
            }

            skeleton::Part::Rotation(value) => {
                let mut new_value = None;
                match value {
                    skeleton::FloatValue::Constant(c) => {
                        ui.label("Rotate by constant");
                        let mut nc = *c;
                        if ui.drag_angle(&mut nc).changed() {
                            new_value = Some(skeleton::FloatValue::Constant(nc));
                        }
                        ui.label("degrees");
                        ui.separator();
                        if ui.small_button("make var").clicked() {
                            new_value = Some(skeleton::FloatValue::Variable("VAR_NAME".into()));
                        }
                    }
                    skeleton::FloatValue::Variable(name) => {
                        ui.label("Rotate by var");
                        let mut nname = name.clone();
                        if ui.text_edit_singleline(&mut nname).changed() {
                            new_value = Some(skeleton::FloatValue::Variable(nname));
                        }
                        ui.label("degrees");
                        ui.separator();
                        if ui.small_button("make const").clicked() {
                            new_value = Some(skeleton::FloatValue::Constant(0.));
                        }
                    }
                }

                if let Some(value) = new_value {
                    changes.push(skeleton::Change::Replace(id, skeleton::Part::Rotation(value)));
                }
            }
            skeleton::Part::EndEffector(name) => {
                let mut nname = name.clone();
                ui.label("End effector named");
                if ui.text_edit_singleline(&mut nname).changed() {
                    changes.push(skeleton::Change::Replace(id, skeleton::Part::EndEffector(nname)));
                }
            }
        }
        ui.separator();
        if ui.small_button("delete").clicked() {
            changes.push(skeleton::Change::Delete(id));
        }
        egui::ComboBox::from_id_source(id)
            .selected_text("add child")
            .show_ui(ui, |ui| {
                if ui.button("Translation").clicked() {
                    changes.push(skeleton::Change::AddToPart(
                        id,
                        skeleton::Part::Translation(0., 0., 0.),
                    ));
                }
                if ui.button("Rotation").clicked() {
                    changes.push(skeleton::Change::AddToPart(
                        id,
                        skeleton::Part::Rotation(skeleton::FloatValue::Constant(0.)),
                    ));
                }
                if ui.button("EndEffector").clicked() {
                    changes.push(skeleton::Change::AddToPart(
                        id,
                        skeleton::Part::EndEffector("EFFECTOR NAME".into()),
                    ));
                }
            });
    });

    let mut children: Vec<_> = md
        .entries
        .iter()
        .filter(|(_, entry)| entry.attached_to == Some(id))
        .collect();
    children.sort_by_key(|(eid, _)| *eid);
    if !children.is_empty() {
        ui.horizontal(|ui| {
            ui.label("   ");
            ui.vertical(|ui| {
                for (child_id, entry) in children {
                    edit_part(ui, *child_id, &entry.part, &md, changes);
                }
            });
        });
    }
}

*/
