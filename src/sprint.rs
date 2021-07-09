use bevy::{app::AppExit, prelude::*, utils::HashMap};
use crate::root_ui::*;


pub struct SprintGame;

impl Plugin for SprintGame {
    fn build(&self, app: &mut AppBuilder) {
        app
            // .see("https://github.com/bevyengine/bevy/issues/69")
            // .require(DefaultPlugins)
            // .require(EguiPlugin)
            // .should_be_implemented()
            .add_startup_system(on_startup.system())
            .add_startup_system(create_menu_entry.system());
    }
}
fn on_startup(){
    info!("Sprint Startup");
}

// TODO: This could be totally generated with a macro
fn create_menu_entry(mut commands: Commands) {
    commands.spawn().insert(MenuEntry {
        name: "Sprint".into(),
        actions: vec![
            MenuEntryAction {
                name: "Start".into(),
                callback: &|cmd: &mut Commands| {
                    cmd.add(SprintCommand::Start);
                },
            }
        ],
    });
}

enum SprintCommand {
    Start,
}

impl bevy::ecs::system::Command for SprintCommand {
    fn write(self: Box<Self>, _world: &mut World) {
        match *self {
            SprintCommand::Start => info!("llegaste aca coño tio"),
        }
    }
}

