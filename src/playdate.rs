use crate::root_ui::*;
use bevy::{app::AppExit, prelude::*, utils::HashMap};

// #[derive(Clone, Copy, Debug)]
// pub enum Pixel {
//     B,
//     W,
//     T,
// }
//
// pub struct Sprite {
//     width: usize,
//     height: usize,
//     origin: Vec2,
//     pixels: Vec<Pixel>,
// }
//
// impl Sprite {
//     pub fn new(width: usize, height: usize) -> Self {
//         Self {
//             width,
//             height,
//             pixels: std::iter::repeat(Pixel::T).take(width*height).collect(),
//         }
//     }
//
//     pub fn width(&self) -> usize {
//         self.width
//     }
//
//     pub fn height(&self) -> usize {
//         self.height
//     }
// }
//
//

pub struct PlaydateSkeletonsPlugin;

impl Plugin for PlaydateSkeletonsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // .require(RootUiPlugin)
            .add_startup_system(on_startup.system())
            .add_startup_system(create_menu_entry.system())
            .insert_resource(SkeletonDatabase::default())
            .add_system(SkeletonDatabase::render_stuff.system())
            .add_system(SkeletonEditor::render_editors.system());
    }
}

fn on_startup(
    mut commands: Commands,
    mut skeleton_db: ResMut<SkeletonDatabase>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    skeleton_db.load_from_disk();
    // commands.spawn_bundle(OrthographicCameraBundle::new_2d());
   
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    }); 
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

// TODO: This could be totally generated with a macro
fn create_menu_entry(mut commands: Commands) {
    commands.spawn().insert(MenuEntry {
        name: "Skeletons".into(),
        actions: vec![
            MenuEntryAction {
                name: "File In".into(),
                callback: &|cmd: &mut Commands| {
                    cmd.add(SkeletonDatabaseCommand::FileIn);
                },
            },
            MenuEntryAction {
                name: "File Out".into(),
                callback: &|cmd: &mut Commands| {
                    cmd.add(SkeletonDatabaseCommand::FileOut);
                },
            },
            MenuEntryAction {
                name: "Open".into(),
                callback: &|cmd: &mut Commands| {
                    cmd.add(SkeletonDatabaseCommand::Open);
                },
            },
            MenuEntryAction {
                name: "Create New".into(),
                callback: &|cmd: &mut Commands| {
                    cmd.add(SkeletonDatabaseCommand::CreateNew);
                },
            },
            MenuEntryAction {
                name: "Preview".into(),
                callback: &|cmd: &mut Commands| {
                    cmd.add(SkeletonDatabaseCommand::Preview);
                },
            },
        ],
    });
}

enum SkeletonDatabaseCommand {
    FileIn,
    FileOut,
    Open,
    CreateNew,
    Preview,
}

impl bevy::ecs::system::Command for SkeletonDatabaseCommand {
    fn write(self: Box<Self>, world: &mut World) {
        let mut skeleton_db = world.get_resource_mut::<SkeletonDatabase>().unwrap();
        match *self {
            SkeletonDatabaseCommand::FileIn => skeleton_db.load_from_disk(),
            SkeletonDatabaseCommand::FileOut => skeleton_db.save_to_disk(),
            SkeletonDatabaseCommand::Open => skeleton_db.open_prompt(),
            SkeletonDatabaseCommand::CreateNew => skeleton_db.create_prompt(),
            SkeletonDatabaseCommand::Preview => skeleton_db.preview_prompt(),
        }
    }
}

#[derive(Debug, Default, Clone)]
struct SkeletonDatabase {
    skeletons: HashMap<String, skeleton::Definition>,
    new_name: Option<String>,
    open_prompt: bool,
    preview_prompt: Option<String>,
}

impl SkeletonDatabase {
    fn create_prompt(&mut self) {
        self.new_name = Some("".into());
    }

    fn preview_prompt(&mut self) {
        if let Some(name) = self.skeletons.keys().next() {
            self.preview_prompt = Some(name.clone());
        }
    }

    fn file_path() -> &'static std::path::Path {
        std::path::Path::new("skeletons.json")
    }

    fn load_from_disk(&mut self) {
        if Self::file_path().exists() {
            info!("File exists, loading!");
            let f = std::fs::File::open(Self::file_path()).expect("Failed to read skeletons file");
            self.skeletons = serde_json::from_reader(f).expect("Failed to parse skeletons file");
            info!("Loaded {} skeletons", self.skeletons.len());
        } else {
            warn!("File does not exist");
        }
    }
    fn save_to_disk(&mut self) {
        info!("Writing to file!");
        let f = std::fs::File::create(Self::file_path()).expect("Failed to write to skeletons file");
        serde_json::to_writer_pretty(f, &self.skeletons).expect("Failed to rialize to skeletons files");
        info!("Wrote {} skeletons", self.skeletons.len());
    }
    fn open_prompt(&mut self) {
        self.open_prompt = true;
    }
    fn render_stuff(
        mut commands: Commands,
        egui_context: ResMut<EguiContext>,
        mut db: ResMut<Self>,
    ) {
        let SkeletonDatabase {
            ref mut skeletons,
            ref mut new_name,
            ref mut open_prompt,
            ref mut preview_prompt,
        } = *db;
        let mut clear_prompt = false;
        if let Some(ref mut new_name) = new_name {
            egui::Window::new("Create New Skeleton").show(egui_context.ctx(), |ui| {
                ui.text_edit_singleline(new_name);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        clear_prompt = true;
                    }
                    if new_name.is_empty() {
                        ui.label("Name cant be empty");
                        return;
                    }
                    if skeletons.contains_key(new_name) {
                        ui.label("Already exists a skeleton with the given name");
                        return;
                    }
                    if ui.button("Create").clicked() {
                        skeletons.insert(new_name.clone(), skeleton::Definition::default());
                        commands
                            .spawn()
                            .insert(SkeletonEditor::for_skeleton(new_name.clone()));
                        clear_prompt = true;
                    }
                });
            });
        }
        if clear_prompt {
            *new_name = None;
        }

        if *open_prompt {
            let mut window_open = true;
            let mut skeleton_to_delete = None;
            egui::Window::new("Skeletons")
                .open(&mut window_open)
                .show(egui_context.ctx(), |ui| {
                    for (name, _skeleton) in skeletons.iter() {
                        ui.horizontal(|ui| {
                            ui.label(name);
                            ui.separator();
                            if ui.small_button("open").clicked() {
                                commands
                                    .spawn()
                                    .insert(SkeletonEditor::for_skeleton(name.clone()));
                                *open_prompt = false;
                            }
                            if ui.small_button("delete").clicked() {
                                skeleton_to_delete = Some(name.clone());
                            }
                        });
                    }
                });
            if let Some(name) = skeleton_to_delete {
                skeletons.remove(&name);
            }
            *open_prompt &= window_open;
        }

        let mut preview_prompt_open = true;
        if let Some(name) = preview_prompt {
            egui::Window::new("Preview Skeleton")
                .open(&mut preview_prompt_open)
                .show(egui_context.ctx(), |ui| {
                    egui::ComboBox::from_label("Skeleton")
                        .selected_text(name.as_str())
                        .show_ui(ui, |ui| {
                            for n in skeletons.keys() {
                                ui.selectable_value(name, n.clone(), n);
                            }
                        });

                    let md = skeletons.get_mut(name).unwrap();

                    let mut vars: Vec<_> = md.variables().into_iter().collect();
                    vars.sort();

                    for var in vars {
                        ui.label(var);
                    }
                });
        }
        if !preview_prompt_open {
            *preview_prompt = None;
        }
    }
}

struct SkeletonEditor {
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

mod skeleton {
    use bevy::utils::{HashMap, HashSet};

    pub struct Instance {
        pub skeleton_name: String,
        pub variables: HashMap<String, f32>,
    }

    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Definition {
        pub next_part_id: usize,
        pub entries: HashMap<usize, PartEntry>,
        // variable_constrains: HashMap<String, VariableConstraint>,
    }

    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
    pub struct VariableConstraint {
        pub min: Option<f32>,
        pub max: Option<f32>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct PartEntry {
        pub attached_to: Option<usize>,
        pub part: Part,
    }

    pub enum Change {
        Delete(usize),
        AddToRoot(Part),
        AddToPart(usize, Part),
        Replace(usize, Part),
    }

    #[derive(Debug, Clone)]
    pub struct PartDoesNotExist(usize);

    impl Definition {
        pub fn apply(&mut self, change: Change) -> Result<(), PartDoesNotExist> {
            match change {
                Change::Delete(id) => {
                    self.entries.remove(&id);
                    for (_, entry) in self.entries.iter_mut() {
                        if entry.attached_to == Some(id) {
                            entry.attached_to = None;
                        }
                    }
                }
                Change::AddToRoot(part) => {
                    let id = self.next_part_id;
                    self.next_part_id += 1;
                    self.entries.insert(
                        id,
                        PartEntry {
                            part,
                            attached_to: None,
                        },
                    );
                }
                Change::AddToPart(pid, part) => {
                    let id = self.next_part_id;
                    self.next_part_id += 1;
                    self.entries.insert(
                        id,
                        PartEntry {
                            part,
                            attached_to: Some(pid),
                        },
                    );
                }
                Change::Replace(id, part) => {
                    self.entries
                        .iter_mut()
                        .find(|(eid, _)| **eid == id)
                        .unwrap()
                        .1
                        .part = part;
                }
            }
            Ok(())
        }

        pub fn variables(&self) -> HashSet<String> {
            let mut set = HashSet::default();
            for entry in self.entries.values() {
                match entry.part {
                    Part::Rotation(FloatValue::Variable(ref var)) => {
                        set.insert(var.clone());
                    }
                    _ => {}
                }
            }
            set
        }
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub enum Part {
        Translation(f32, f32, f32),
        Rotation(FloatValue),
        EndEffector(String),
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub enum FloatValue {
        Variable(String),
        Constant(f32),
    }
}
