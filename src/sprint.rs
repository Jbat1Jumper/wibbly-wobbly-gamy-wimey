use bevy::{app::AppExit, prelude::*, utils::HashMap};
use bevy_egui::{egui, EguiContext, EguiPlugin};


pub struct SprintGame;

impl Plugin for SprintGame {
    fn build(&self, app: &mut AppBuilder) {
        app
            // .see("https://github.com/bevyengine/bevy/issues/69")
            // .require(DefaultPlugins)
            // .require(EguiPlugin)
            // .should_be_implemented()
            .add_startup_system(on_startup.system())
            .add_system(editor_ui.system());
    }
}
fn on_startup(){
    info!("Sprint Startup");
}

fn editor_ui(
    egui_context: ResMut<EguiContext>,
) {
    egui::TopPanel::top("sprint_menu").show(egui_context.ctx(), |ui| {
        egui::menu::bar(ui, |ui| {
            egui::menu::menu(ui, "Sprint", |ui| {
                if ui.button("Hi").clicked() {
                    info!("You got here!");
                }
            });
        });
    });
}
