#[macro_use]
mod common;
mod plain_simple_physics;
mod playdate;
mod pyxel_plugin;
mod sprint;
mod peach;
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
        .add_plugin(root_ui::RootUiPlugin)
        .add_plugin(playdate::PlaydateSkeletonsPlugin)
        //.add_plugin(sprint::SprintGame)
        //.add_plugin(peach::PeachyThingiesPlugin)
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

mod root_ui {
    use bevy::app::AppExit;

    // Export for usability and strangling dependences
    use bevy::prelude::*;
    pub use bevy_egui::{egui, EguiContext};

    pub struct RootUiPlugin;

    impl Plugin for RootUiPlugin {
        fn build(&self, app: &mut AppBuilder) {
            app
                // .see("https://github.com/bevyengine/bevy/issues/69")
                // .require(DefaultPlugins)
                // .require(EguiPlugin)
                // .should_be_implemented()
                .add_system(draw_root_menu.system())
                .add_plugin(file_menu::FileMenuPlugin);
        }
    }

    /// Each MenuEntry represents a dropdown menu with actions
    /// in the main menu bar, alongsides the always
    /// present `File` one.
    ///
    // TODO: Add ordering to the entries
    pub struct MenuEntry {
        pub name: String,
        pub actions: Vec<MenuEntryAction>,
    }

    /// When pressed, it triggers some logic only known to the
    /// owner of the menu entry
    ///
    pub struct MenuEntryAction {
        pub name: String,
        pub callback: &'static dyn ActionCallback,
    }

    pub trait ActionCallback: Fn(&mut Commands) -> () + Sync {}
    impl<F> ActionCallback for F where F: Fn(&mut Commands) -> () + Sync {}

    mod file_menu {
        use super::*;
        use bevy::{app, prelude::*};

        pub struct FileMenuPlugin;

        impl Plugin for FileMenuPlugin {
            fn build(&self, app: &mut AppBuilder) {
                app
                    // .require(RootUiPlugin)
                    .add_startup_system(create_file_menu_entry.system());
            }
        }

        fn create_file_menu_entry(mut commands: Commands) {
            commands.spawn().insert(MenuEntry {
                name: "File".into(),
                actions: vec![MenuEntryAction {
                    name: "Quit".into(),
                    callback: &|cmd: &mut Commands| {
                        cmd.add(FileCommand::Quit);
                    },
                }],
            });
        }

        enum FileCommand {
            Quit,
        }

        impl bevy::ecs::system::Command for FileCommand {
            fn write(self: Box<Self>, world: &mut World) {
                match *self {
                    FileCommand::Quit => {
                        let mut quit_events = world
                            .get_resource_mut::<bevy::app::Events<app::AppExit>>()
                            .unwrap();
                        quit_events.send(AppExit);
                    }
                }
            }
        }
    }

    fn draw_root_menu(
        mut commands: Commands,
        egui_context: ResMut<EguiContext>,
        menu_entries: Query<&MenuEntry>,
    ) {
        egui::TopPanel::top("root_menu").show(egui_context.ctx(), |ui| {
            egui::menu::bar(ui, |ui| {
                let mut menu_entries: Vec<_> = menu_entries.iter().collect();
                menu_entries.sort_by(|a, b| a.name.cmp(&b.name));
                for entry in menu_entries {
                    egui::menu::menu(ui, &entry.name, |ui| {
                        for action in entry.actions.iter() {
                            if ui.button(&action.name).clicked() {
                                (action.callback)(&mut commands);
                            }
                        }
                    });
                }
            });
        });
    }
}
