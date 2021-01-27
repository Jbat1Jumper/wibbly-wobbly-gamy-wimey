use ggez;
use glam::f32::Vec2;

use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};
use std::env;
use std::path;

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

fn create_main_menu(state: &mut MainState) 
{
    state.nodes.create();

    let banner = state.nodes.create();
    state.texts.insert(banner, graphics::Text::new(("Hello", state.font, 48.0)));
    state.positions.insert(banner, Vec2::new(10.0, 10.0));

    let banner2 = state.nodes.create();
    state.texts.insert(banner2, graphics::Text::new(("          world!", state.font, 48.0)));
    state.positions.insert(banner2, Vec2::new(10.0, 80.0));
}

struct MainState {
    frames: usize,
    font: graphics::Font,
    nodes: NodeSet,
    texts: HashMap<Nid, graphics::Text>,
    positions: HashMap<Nid, Vec2>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let font = graphics::Font::new(ctx, "/LiberationMono-Regular.ttf")?;

        Ok(MainState {
            font,
            frames: 0,
            nodes: NodeSet {
                nodes: HashSet::new(),
                last_id: 0,
            },

            positions: HashMap::new(),
            texts: HashMap::new(),
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        for node in self.nodes.nodes.iter() {
            if let (
                Some(text),
                Some(position),
            ) = (
                self.texts.get(node),
                self.positions.get(node),
            ) {
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

    let mut state = MainState::new(&mut ctx)?;
    create_main_menu(&mut state);
    event::run(&mut ctx, &mut event_loop, &mut state)
}
