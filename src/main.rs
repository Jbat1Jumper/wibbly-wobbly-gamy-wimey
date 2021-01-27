use ggez;
use glam::f32::Vec2;

use ggez::event::{EventHandler, run as launch};
use ggez::graphics;
use ggez::{Context, GameResult};
use std::env;
use std::path;
use std::time::Duration;

use std::sync::mpsc::{channel, Receiver, Sender};

use std::collections::{HashMap, HashSet};

type Nid = i32;

trait NodeId {
    fn next(&self) -> Self;
}

impl NodeId for Nid {
    fn next(&self) -> Nid {
        self + 1
    }
}

struct NodeSet {
    nodes: HashSet<Nid>,
    last_id: Nid,
}

impl NodeSet {
    fn create(&mut self) -> Nid {
        let new_id = self.last_id.next();
        self.nodes.insert(new_id);
        self.last_id = new_id;
        new_id
    }
    fn destroy(&mut self, node: Nid) -> bool {
        self.nodes.remove(&node)
    }
    fn exists(&self, node: Nid) -> bool {
        self.nodes.contains(&node)
    }
}

struct State {
    scene: Scene,
    event_bus: Receiver<Event>,
    event_bus_sender: Sender<Event>,
}

enum Scene {
    Intro(Intro),
    MainMenu(MainMenu),
}

enum Event {
    IntroEnded,
}

struct Intro {
    remaining: Duration,
    sender: Sender<Event>,
}

impl Intro {
    fn new(ctx: &mut Context, sender: Sender<Event>) -> GameResult<Intro> {
        Ok(Intro { remaining: Duration::from_secs(1), sender })
    }
}

struct IntroEnded;

struct MainMenu {
    frames: usize,
    font: graphics::Font,
    nodes: NodeSet,
    texts: HashMap<Nid, graphics::Text>,
    positions: HashMap<Nid, Vec2>,
}

impl MainMenu {
    fn new(ctx: &mut Context) -> GameResult<MainMenu> {
        let mut m = MainMenu {
            font: graphics::Font::new(ctx, "/LiberationMono-Regular.ttf")?,
            frames: 0,
            nodes: NodeSet {
                nodes: HashSet::new(),
                last_id: 0,
            },
            positions: HashMap::new(),
            texts: HashMap::new(),
        };
        m.init();
        Ok(m)
    }

    fn init(&mut self) {
        let banner = self.nodes.create();
        self.texts
            .insert(banner, graphics::Text::new(("Hello", self.font, 48.0)));
        self.positions.insert(banner, Vec2::new(10.0, 10.0));

        let banner2 = self.nodes.create();
        self.texts.insert(
            banner2,
            graphics::Text::new(("          world!", self.font, 48.0)),
        );
        self.positions.insert(banner2, Vec2::new(10.0, 80.0));
    }
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> {
        let (event_bus_sender, event_bus) = channel();
        Ok(State {
            scene: Scene::Intro(Intro::new(ctx, event_bus_sender.clone())?),
            event_bus,
            event_bus_sender,
        })
    }

    fn update_scenes(&mut self, ctx: &mut Context) -> GameResult {
        match &mut self.scene {
            Scene::Intro(intro) => intro.update(ctx),
            Scene::MainMenu(main_menu) => main_menu.update(ctx),
        }
    }

    fn handle_events(&mut self, ctx: &mut Context) -> GameResult {
        for e in self.event_bus.try_iter() {
            match e {
                Event::IntroEnded => {
                    self.scene = Scene::MainMenu(MainMenu::new(ctx)?);
                }
            }
        }
        Ok(())
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.update_scenes(ctx)?;
        self.handle_events(ctx)?;
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        match &mut self.scene {
            Scene::Intro(intro) => intro.draw(ctx),
            Scene::MainMenu(main_menu) => main_menu.draw(ctx),
        }
    }
}

impl EventHandler for Intro {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let delta = ggez::timer::delta(ctx);
        if self.remaining > delta {
            self.remaining -= delta;
        } else {
            self.sender.send(Event::IntroEnded);
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        Ok(())
    }
}

impl EventHandler for MainMenu {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        for node in self.nodes.nodes.iter() {
            if let (Some(text), Some(position)) = (self.texts.get(node), self.positions.get(node)) {
                graphics::draw(ctx, text, (position.clone(),))?;
            }
        }
        graphics::present(ctx)?;

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::fps(ctx));
        }

        Ok(())
    }
}

pub fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("helloworld", "ggez").add_resource_path(resource_dir);
    let (mut ctx, mut event_loop) = cb.build()?;

    let mut state = State::new(&mut ctx)?;

    launch(&mut ctx, &mut event_loop, &mut state)
}
