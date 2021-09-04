use crate::root_ui::*;
use bevy::{prelude::*, utils::HashMap};

use self::{
    mursten::InMemoryModel, mursten_bevy_plugin::CurrentModel, mursten_egui_editor::ModelEditor,
};

mod mursten;
mod mursten_bevy_plugin;
mod mursten_egui_editor;
mod skeleton;
mod skeleton_editor;
mod skeleton_instance;

// use skeleton_editor::SkeletonEditor;

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
            .insert_resource(CurrentModel(Box::new(mursten::example::and_one_more_is_five_model())))
            .insert_resource(ModelEditor::default())
            .add_system(mursten_model_editor.system())
            .add_startup_system(on_startup.system())
            .add_startup_system(create_menu_entry.system())
            .insert_resource(SkeletonDatabase::default())
            .add_system(SkeletonDatabase::render_stuff.system());
        //.add_system(SkeletonEditor::render_editors.system());
    }
}

fn mursten_model_editor(
    mut model: ResMut<CurrentModel>,
    mut editor: ResMut<ModelEditor>,
    mut egui_context: ResMut<EguiContext>,
) {
    egui::Window::new(editor.title()).id(bevy_egui::egui::Id::new("Mursten editor")).show(egui_context.ctx(), |ui| {
        editor.show(&*model, ui);
        editor.fullfill_actions(&mut *model);
    });
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
        let f =
            std::fs::File::create(Self::file_path()).expect("Failed to write to skeletons file");
        serde_json::to_writer_pretty(f, &self.skeletons)
            .expect("Failed to rialize to skeletons files");
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
                    // if ui.button("Create").clicked() {
                    //     skeletons.insert(new_name.clone(), skeleton::Definition::default());
                    //     commands
                    //         .spawn()
                    //         .insert(SkeletonEditor::for_skeleton(new_name.clone()));
                    //     clear_prompt = true;
                    // }
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
                            // if ui.small_button("open").clicked() {
                            //     commands
                            //         .spawn()
                            //         .insert(SkeletonEditor::for_skeleton(name.clone()));
                            //     *open_prompt = false;
                            // }
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
