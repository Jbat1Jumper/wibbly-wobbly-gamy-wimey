use bevy::prelude::*;

pub mod game;
pub mod intro;
pub mod main_menu;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum UnScene {
    Intro,
    MainMenu,
    Game,
    Exit,
}

pub struct LoadGameScenes;

impl Plugin for LoadGameScenes {
    fn build(&self, app: &mut AppBuilder) {
        app.add_state(UnScene::Intro)
            .add_plugin(intro::Intro)
            .add_plugin(main_menu::MainMenu)
            .add_plugin(game::GameScene)
            .add_system_set(
                SystemSet::on_enter(UnScene::Exit).with_system(death_of_it_all.system()),
            );
    }
}

fn death_of_it_all() {
    info!("All good, all ok, this is graceful shutdown now...");
    std::process::exit(0);
}
