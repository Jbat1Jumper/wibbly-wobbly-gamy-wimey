use bevy::{prelude::*, utils::HashMap};
use crate::root_ui::*;


pub struct PeachyThingiesPlugin;

impl Plugin for PeachyThingiesPlugin {
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
    info!("Peach Startup");
}

// TODO: This could be totally generated with a macro 
fn create_menu_entry(mut commands: Commands) {
    commands.spawn().insert(MenuEntry {
        name: "Peach".into(),
        actions: vec![
            MenuEntryAction {
                name: "Start".into(),
                callback: &|cmd: &mut Commands| {
                    cmd.add(Command::Start);
                },
            }
        ],
    });
}

enum Command {
    Start,
}

impl bevy::ecs::system::Command for Command {
    fn write(self: Box<Self>, _world: &mut World) {
        match *self {
            Command::Start => info!("llegaste aca coño tio"),
        }
    }
}

