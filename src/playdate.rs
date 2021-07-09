use bevy::{app::AppExit, prelude::*, utils::HashMap};
use bevy_egui::{egui, EguiContext, EguiPlugin};

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

pub struct PlaydateModelsPlugin;

impl Plugin for PlaydateModelsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // .see("https://github.com/bevyengine/bevy/issues/69")
            // .require(DefaultPlugins)
            // .require(EguiPlugin)
            // .should_be_implemented()
            .add_startup_system(on_startup.system())
            .add_system(editor_ui.system())
            .insert_resource(ModelDatabase::default())
            .add_system(ModelDatabase::render_stuff.system())
            .add_system(ModelEditor::render_editors.system());
    }
}

fn on_startup(mut commands: Commands, mut model_db: ResMut<ModelDatabase>) {
    model_db.load_from_disk();
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn editor_ui(
    egui_context: ResMut<EguiContext>,
    mut model_db: ResMut<ModelDatabase>,
) {
    egui::TopPanel::top("playdate_menu").show(egui_context.ctx(), |ui| {
        egui::menu::bar(ui, |ui| {
            egui::menu::menu(ui, "Models", |ui| {
                if ui.button("File In").clicked() {
                    model_db.load_from_disk();
                }
                if ui.button("File Out").clicked() {
                    model_db.save_to_disk();
                }
                ui.separator();
                if ui.button("Open").clicked() {
                    model_db.open_prompt();
                }
                if ui.button("Create New").clicked() {
                    model_db.create_prompt();
                }
                if ui.button("Preview").clicked() {
                    model_db.preview_prompt();
                }
            });
        });
    });
}

#[derive(Debug, Default, Clone)]
struct ModelDatabase {
    models: HashMap<String, model::Definition>,
    new_name: Option<String>,
    open_prompt: bool,
    preview_prompt: Option<String>,
}

impl ModelDatabase {
    fn create_prompt(&mut self) {
        self.new_name = Some("".into());
    }

    fn preview_prompt(&mut self) {
        if let Some(name) = self.models.keys().next() {
            self.preview_prompt = Some(name.clone());
        }
    }

    fn file_path() -> &'static std::path::Path {
        std::path::Path::new("models.json")
    }

    fn load_from_disk(&mut self) {
        if Self::file_path().exists() {
            info!("File exists, loading!");
            let f = std::fs::File::open(Self::file_path()).expect("Failed to read models file");
            self.models = serde_json::from_reader(f).expect("Failed to parse models file");
            info!("Loaded {} models", self.models.len());
        } else {
            warn!("File does not exist");
        }
    }
    fn save_to_disk(&mut self) {
        info!("Writing to file!");
        let f = std::fs::File::create(Self::file_path()).expect("Failed to write to models file");
        serde_json::to_writer_pretty(f, &self.models).expect("Failed to rialize to models files");
        info!("Wrote {} models", self.models.len());
    }
    fn open_prompt(&mut self) {
        self.open_prompt = true;
    }
    fn render_stuff(
        mut commands: Commands,
        egui_context: ResMut<EguiContext>,
        mut db: ResMut<Self>,
    ) {
        let ModelDatabase {
            ref mut models,
            ref mut new_name,
            ref mut open_prompt,
            ref mut preview_prompt,
        } = *db;
        let mut clear_prompt = false;
        if let Some(ref mut new_name) = new_name {
            egui::Window::new("Create New Model").show(egui_context.ctx(), |ui| {
                ui.text_edit_singleline(new_name);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        clear_prompt = true;
                    }
                    if new_name.is_empty() {
                        ui.label("Name cant be empty");
                        return;
                    }
                    if models.contains_key(new_name) {
                        ui.label("Already exists a model with the given name");
                        return;
                    }
                    if ui.button("Create").clicked() {
                        models.insert(new_name.clone(), model::Definition::default());
                        commands
                            .spawn()
                            .insert(ModelEditor::for_model(new_name.clone()));
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
            let mut model_to_delete = None;
            egui::Window::new("Models")
                .open(&mut window_open)
                .show(egui_context.ctx(), |ui| {
                    for (name, _model) in models.iter() {
                        ui.horizontal(|ui| {
                            ui.label(name);
                            ui.separator();
                            if ui.small_button("open").clicked() {
                                commands
                                    .spawn()
                                    .insert(ModelEditor::for_model(name.clone()));
                                *open_prompt = false;
                            }
                            if ui.small_button("delete").clicked() {
                                model_to_delete = Some(name.clone());
                            }
                        });
                    }
                });
            if let Some(name) = model_to_delete {
                models.remove(&name);
            }
            *open_prompt &= window_open;
        }

        let mut preview_prompt_open = true;
        if let Some(name) = preview_prompt {
            egui::Window::new("Preview Model")
                .open(&mut preview_prompt_open)
                .show(egui_context.ctx(), |ui| {
                    egui::ComboBox::from_label("Model")
                        .selected_text(name.as_str())
                        .show_ui(ui, |ui| {
                            for n in models.keys() {
                                ui.selectable_value(name, n.clone(), n);
                            }
                        });

                    let md = models.get_mut(name).unwrap();

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

struct ModelEditor {
    model_name: String,
}

impl ModelEditor {
    pub fn for_model(model_name: String) -> Self {
        Self { model_name }
    }

    pub fn render_editors(
        mut commands: Commands,
        egui_context: ResMut<EguiContext>,
        mut editors: Query<(Entity, &mut ModelEditor)>,
        mut model_db: ResMut<ModelDatabase>,
    ) {
        for (editor_eid, mut editor) in editors.iter_mut() {
            let mut open = true;
            egui::Window::new(format!("Model Editor: {}", editor.model_name))
                .open(&mut open)
                .show(egui_context.ctx(), |ui| {
                    let md = model_db.models.get_mut(&editor.model_name).unwrap();

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
                                changes.push(model::Change::AddToRoot(model::Part::Translation(
                                    0., 0., 0.,
                                )));
                            }
                            if ui.button("Rotation").clicked() {
                                changes.push(model::Change::AddToRoot(model::Part::Rotation(
                                    model::FloatValue::Constant(0.),
                                )));
                            }
                            if ui.button("EndEffector").clicked() {
                                changes.push(model::Change::AddToRoot(model::Part::EndEffector(
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

fn part_name(part: &model::Part) -> String {
    match part {
        model::Part::Translation(x, y, z) => format!("Translation  ({}, {}, {})", x, y, z),
        model::Part::Rotation(value) => match value {
            model::FloatValue::Constant(c) => format!("Rotation by {} deg", c),
            model::FloatValue::Variable(name) => format!("Rotation by {} deg", name),
        },
        model::Part::EndEffector(name) => format!("EndEffector {}", name),
    }
}

fn edit_part(
    ui: &mut egui::Ui,
    id: usize,
    part: &model::Part,
    md: &model::Definition,
    changes: &mut Vec<model::Change>,
) {
    ui.horizontal(|ui| {
        ui.label("▶");
        match part {
            model::Part::Translation(x, y, z) => {
                let mut p = (*x, *y, *z);
                ui.label("Translate by (");
                ui.add(egui::DragValue::new(&mut p.0).speed(0.1));
                ui.add(egui::DragValue::new(&mut p.1).speed(0.1));
                ui.add(egui::DragValue::new(&mut p.2).speed(0.1));
                ui.label(")");

                if p != (*x, *y, *z) {
                    changes.push(model::Change::Replace(
                        id,
                        model::Part::Translation(p.0, p.1, p.2),
                    ));
                }
            }

            model::Part::Rotation(value) => {
                let mut new_value = None;
                match value {
                    model::FloatValue::Constant(c) => {
                        ui.label("Rotate by constant");
                        let mut nc = *c;
                        if ui.drag_angle(&mut nc).changed() {
                            new_value = Some(model::FloatValue::Constant(nc));
                        }
                        ui.label("degrees");
                        ui.separator();
                        if ui.small_button("make var").clicked() {
                            new_value = Some(model::FloatValue::Variable("VAR_NAME".into()));
                        }
                    }
                    model::FloatValue::Variable(name) => {
                        ui.label("Rotate by var");
                        let mut nname = name.clone();
                        if ui.text_edit_singleline(&mut nname).changed() {
                            new_value = Some(model::FloatValue::Variable(nname));
                        }
                        ui.label("degrees");
                        ui.separator();
                        if ui.small_button("make const").clicked() {
                            new_value = Some(model::FloatValue::Constant(0.));
                        }
                    }
                }

                if let Some(value) = new_value {
                    changes.push(model::Change::Replace(id, model::Part::Rotation(value)));
                }
            }
            model::Part::EndEffector(name) => {
                let mut nname = name.clone();
                ui.label("End effector named");
                if ui.text_edit_singleline(&mut nname).changed() {
                    changes.push(model::Change::Replace(id, model::Part::EndEffector(nname)));
                }
            }
        }
        ui.separator();
        if ui.small_button("delete").clicked() {
            changes.push(model::Change::Delete(id));
        }
        egui::ComboBox::from_id_source(id)
            .selected_text("add child")
            .show_ui(ui, |ui| {
                if ui.button("Translation").clicked() {
                    changes.push(model::Change::AddToPart(
                        id,
                        model::Part::Translation(0., 0., 0.),
                    ));
                }
                if ui.button("Rotation").clicked() {
                    changes.push(model::Change::AddToPart(
                        id,
                        model::Part::Rotation(model::FloatValue::Constant(0.)),
                    ));
                }
                if ui.button("EndEffector").clicked() {
                    changes.push(model::Change::AddToPart(
                        id,
                        model::Part::EndEffector("EFFECTOR NAME".into()),
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

mod model {
    use bevy::utils::{HashMap, HashSet};

    pub struct Instance {
        pub model_name: String,
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
