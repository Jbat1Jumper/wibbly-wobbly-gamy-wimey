use ggez;
use glam::f32::Vec2;

use ggez::event::{run as launch, EventHandler};
use ggez::graphics;
use ggez::GameResult;
use std::env;
use std::path;
use std::time::Duration;

use std::sync::mpsc::{channel, Receiver, Sender};

use std::collections::{HashMap, HashSet};

use legion::*;

use pyxel::Pyxel;

mod components;
use components::*;

use Direction::*;
use Constrain::*;
use Rotation::*;

#[derive(Clone, Copy, Debug)]
enum Command {
    GoToMainMenu,
    StartNewGame,
    Exit,
}

#[derive(Clone, Copy, Debug)]
enum Sprite {
    TileRef(usize, TilesetRef),
}

#[derive(Clone, Copy, Debug)]
struct SpriteTransform {
    rotation: Rotation,
    flipped: bool,
}

#[derive(Clone, Debug)]
struct TileMap {
    tiles: Vec<Vec<Tile>>,
    size: (usize, usize),
}

impl TileMap {
    fn new(size: (usize, usize)) -> TileMap {
        TileMap {
            size,
            tiles: std::iter::repeat_with(|| std::iter::repeat(Tile::Empty).take(size.1).collect())
                .take(size.0)
                .collect(),
        }
    }

    fn at(&self, pos: (usize, usize)) -> Tile {
        if self.in_bounds(pos) {
            self.tiles[pos.1][pos.0]
        } else {
            Tile::Empty
        }
    }

    fn in_bounds(&self, pos: (usize, usize)) -> bool {
        panic!()
    }

    #[rustfmt::skip]
    fn neighborhood(&self, pos: (usize, usize)) -> [Tile; 9] {
        [
            self.at(pos.step(Left).step(Up)),   self.at(pos.step(Up)),   self.at(pos.step(Right).step(Up)),
            self.at(pos.step(Left)),            self.at(pos),            self.at(pos.step(Right)),
            self.at(pos.step(Left).step(Down)), self.at(pos.step(Down)), self.at(pos.step(Right).step(Down)),
        ]
    }
}

trait GridWalkable {
    fn step(&self, direction: Direction) -> Self;
}

impl GridWalkable for (usize, usize) {
    fn step(&self, direction: Direction) -> Self {
        let (x, y) = *self;
        match direction {
            Direction::Up => (x, y - 1),
            Direction::Left => (x - 1, y),
            Direction::Down => (x, y + 1),
            Direction::Right => (x + 1, y),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Rotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Rotation {
    fn all() -> [Rotation; 4] {
        [
            Deg0,
            Deg90,
            Deg180,
            Deg270,
        ]
    }
}

trait Rotable {
    fn rotate(self, rotation: &Rotation) -> Self;
}

trait Flippable {
    fn flip_horizontally(self) -> Self;
}

impl<T> Rotable for [T; 9]
where 
    T: Copy,
{
    #[rustfmt::skip]
    fn rotate(self, rotation: &Rotation) -> [T; 9] {
        match rotation {
            Deg0 => self,
            Deg90 => [
                self[6], self[3], self[0], 
                self[7], self[4], self[1], 
                self[8], self[5], self[2], 
            ],
            Deg180 => [
                self[8], self[7], self[6], 
                self[5], self[4], self[3], 
                self[2], self[1], self[0], 
            ],
            Deg270 => [
                self[2], self[5], self[8], 
                self[1], self[4], self[7], 
                self[0], self[3], self[6], 
            ],
        }
    }
}

impl<T> Flippable for [T; 9] 
where 
    T: Copy,
{
    #[rustfmt::skip]
    fn flip_horizontally(self) -> [T; 9] {
        [
            self[2], self[1], self[0], 
            self[5], self[4], self[3], 
            self[8], self[7], self[6], 
        ]
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Empty,
    Ground,
    Wall,
    Door(DoorKind, DoorState, Direction),
}

fn get_tile_sprite_and_transform (
    pos: (usize, usize),
    tile_map: &TileMap,
    tileset: Tileset
) -> Option<( Sprite, SpriteTransform)> {
    let nh = tile_map.neighborhood(pos);

    for (id, constrains) in tileset.tiles.iter() {
        for flipped in &[false, true] {
            let constrains = if *flipped { constrains.clone().flip_horizontally() } else { constrains.clone() };
            for rotation in &Rotation::all() {
                let constrains = constrains.clone().rotate(rotation);


                let fits = constrains.iter()
                    .zip(nh.iter())
                    .all(|(constrain, tile)| constrain.satisfies(tile) );

                return Some((
                    Sprite::TileRef(*id, tileset.reference()), 
                    SpriteTransform {
                        rotation: *rotation,
                        flipped: *flipped,
                    },
                ));

            }
        }
    }
    None
}


#[derive(Clone, Copy, Debug)]
struct Frame(u32);

#[derive(Clone, Copy, Debug)]
enum TilesetRef {
    PyxelFile(&'static str),
}

struct Tileset {
    pyxel_file: &'static str,
    tiles: HashMap<usize, [Constrain<Tile>; 9]>,
}

impl Tileset {
    fn reference(&self) -> TilesetRef {
        TilesetRef::PyxelFile(self.pyxel_file)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DoorState {
    Open,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DoorKind {
    Known,
    Unknown,
    Dark,
}

#[derive(Clone, Debug)]
struct Rng;

#[derive(Clone, Debug)]
struct RoomParams {
    connection_constrains: HashMap<Direction, Constrain<Connection>>,
}

#[derive(Clone, Copy, Debug)]
enum Constrain<T> {
    Unrestricted,
    MustBe(T),
}

impl<T> Constrain<T>
where
    T: Eq,
{
    fn satisfies(&self, item: &T) -> bool {
        match self {
            Unrestricted => true,
            MustBe(something) => something == item,
        }
    }
}

#[derive(Clone, Debug)]
struct Room {
    size: (usize, usize),
    tile_map: TileMap,
    connections: HashMap<Direction, Connection>,
}

#[derive(Clone, Copy, Debug)]
enum Connection {
    Common,
    Dark,
    NotConnected,
}

trait RoomCreator {
    fn create_room(params: &RoomParams, rng: &mut Rng) -> Room;
    fn populate(&mut self, params: &RoomParams, rng: &mut Rng);
}

struct SimpleRoomCreator;

impl RoomCreator for SimpleRoomCreator {
    fn create_room(params: &RoomParams, rng: &mut Rng) -> Room {
        let mut connections = HashMap::new();

        for (dir, constrain) in params.connection_constrains.iter() {
            connections.insert(
                *dir,
                match constrain {
                    Unrestricted => Connection::Common,
                    MustBe(con) => *con,
                },
            );
        }
        Room {
            connections,
            size: (8, 5),
            tile_map: TileMap::new((8, 5)),
        }
    }

    fn populate(&mut self, params: &RoomParams, rng: &mut Rng) {}
}

#[derive(Clone, Copy, Debug)]
struct Position(Vec2);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Text {
    string: String,
    font: Font,
    size: usize,
}

impl Text {
    fn new<T: Into<String>>(s: T, font: Font, size: usize) -> Text {
        Text {
            string: s.into(),
            font,
            size,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Font {
    LiberationMono,
}

impl Font {
    fn resource_path(&self) -> &'static str {
        match self {
            Font::LiberationMono => "/LiberationMono-Regular.ttf",
        }
    }
}

struct Context {
    ggez_ctx: ggez::Context,
    ggez_events_loop: ggez::event::EventsLoop,
    text_resources: TextResources,
    command_bus: Vec<Command>,
    frames: usize,
}

impl Context {
    fn draw_text(&mut self, text: &Text, position: &Position) -> GameResult {
        let Position(pos) = position;
        let rtext = self.text_resources.render_text(&mut self.ggez_ctx, text)?;
        graphics::draw(&mut self.ggez_ctx, rtext, (pos.clone(),))
    }

    fn queue(&mut self, cmd: Command) {
        self.command_bus.push(cmd);
    }

    fn delta_time(&mut self) -> Duration {
        ggez::timer::delta(&self.ggez_ctx)
    }

    fn quit(&mut self) {
        ggez::event::quit(&mut self.ggez_ctx);
    }
}

struct TextResources {
    rendered_texts: HashMap<Text, graphics::Text>,
    loaded_fonts: HashMap<Font, graphics::Font>,
}

impl TextResources {
    fn get_font<'a>(
        &'a mut self,
        ctx: &mut ggez::Context,
        font: &Font,
    ) -> GameResult<&'a graphics::Font> {
        if !self.loaded_fonts.contains_key(font) {
            let gfont = graphics::Font::new(ctx, font.resource_path())?;
            self.loaded_fonts.insert(*font, gfont);
        }
        Ok(self.loaded_fonts.get(font).unwrap())
    }

    fn render_text<'a>(
        &'a mut self,
        ctx: &mut ggez::Context,
        text: &Text,
    ) -> GameResult<&'a graphics::Text> {
        if !self.rendered_texts.contains_key(text) {
            let Text { string, font, size } = text;
            let gfont = self.get_font(ctx, font)?;
            let rtext = graphics::Text::new((string.clone(), *gfont, *size as f32));
            self.rendered_texts.insert(text.clone(), rtext);
        }
        Ok(self.rendered_texts.get(text).unwrap())
    }
}

enum Button {
    Start,
    A,
    B,
    Left,
    Right,
    Up,
    Down,
}

enum ButtonState {
    Pressed,
    Released,
}

struct Game {
    scene: Scene,
    context: Context,
}

impl Game {
    fn new(
        (ggez_ctx, ggez_events_loop): (ggez::Context, ggez::event::EventsLoop),
    ) -> GameResult<Game> {
        Ok(Game {
            scene: Scene::Intro(Intro::new()),
            context: Context {
                ggez_ctx,
                ggez_events_loop,
                frames: 0,
                text_resources: TextResources {
                    loaded_fonts: HashMap::new(),
                    rendered_texts: HashMap::new(),
                },
                command_bus: vec![],
            },
        })
    }

    fn key_binding(key: ggez::input::keyboard::KeyCode) -> Option<Button> {
        use ggez::input::keyboard::KeyCode;
        match key {
            KeyCode::Escape => Some(Button::Start),
            KeyCode::W => Some(Button::Up),
            KeyCode::A => Some(Button::Left),
            KeyCode::S => Some(Button::Down),
            KeyCode::D => Some(Button::Right),
            KeyCode::J => Some(Button::A),
            KeyCode::K => Some(Button::B),
            _ => None,
        }
    }

    fn state_binding(state: ggez::event::winit_event::ElementState) -> ButtonState {
        match state {
            ggez::event::winit_event::ElementState::Pressed => ButtonState::Pressed,
            ggez::event::winit_event::ElementState::Released => ButtonState::Released,
        }
    }

    fn run(&mut self) -> GameResult {
        while self.context.ggez_ctx.continuing {
            let mut button_events: Vec<(Button, ButtonState)> = vec![];

            {
                let Context {
                    ggez_ctx,
                    ggez_events_loop,
                    ..
                } = &mut self.context;
                ggez_ctx.timer_context.tick();
                ggez_events_loop.poll_events(|event| {
                    ggez_ctx.process_event(&event);
                    match event {
                        ggez::event::winit_event::Event::WindowEvent { event, .. } => match event {
                            ggez::event::winit_event::WindowEvent::CloseRequested => {
                                ggez::event::quit(ggez_ctx)
                            }
                            ggez::event::winit_event::WindowEvent::KeyboardInput {
                                input:
                                    ggez::event::winit_event::KeyboardInput {
                                        virtual_keycode: Some(keycode),
                                        state,
                                        ..
                                    },
                                ..
                            } => match Self::key_binding(keycode) {
                                Some(button) => {
                                    button_events.push((button, Self::state_binding(state)))
                                }
                                None => (),
                            },
                            // `CloseRequested` and `KeyboardInput` events won't appear here.
                            x => (), //println!("Other window event fired: {:?}", x),
                        },

                        x => (), //println!("Device event fired: {:?}", x),
                    }
                });
            }

            for (button, state) in button_events.iter() {
                self.scene_on_input(button, state);
            }

            self.update_scene()?;
            self.handle_events()?;
            self.draw_scene()?;
        }
        Ok(())
    }

    fn scene_on_input(&mut self, button: &Button, state: &ButtonState) -> GameResult {
        match &mut self.scene {
            Scene::Intro(intro) => intro.on_input(&mut self.context, button, state),
            Scene::MainMenu(main_menu) => main_menu.on_input(&mut self.context, button, state),
            Scene::GameScene(s) => s.on_input(&mut self.context, button, state),
        }
    }

    fn update_scene(&mut self) -> GameResult {
        match &mut self.scene {
            Scene::Intro(intro) => intro.update(&mut self.context),
            Scene::MainMenu(main_menu) => main_menu.update(&mut self.context),
            Scene::GameScene(s) => s.update(&mut self.context),
        }
    }

    fn draw_scene(&mut self) -> GameResult {
        graphics::clear(&mut self.context.ggez_ctx, [0.1, 0.2, 0.3, 1.0].into());

        match &mut self.scene {
            Scene::Intro(intro) => intro.draw(&mut self.context),
            Scene::MainMenu(main_menu) => main_menu.draw(&mut self.context),
            Scene::GameScene(s) => s.draw(&mut self.context),
        };

        graphics::present(&mut self.context.ggez_ctx)?;

        self.context.frames += 1;
        if (self.context.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::fps(&self.context.ggez_ctx));
        }

        Ok(())
    }

    fn handle_events(&mut self) -> GameResult {
        for e in self.context.command_bus.iter() {
            match e {
                Command::GoToMainMenu => {
                    self.scene = Scene::MainMenu(MainMenu::new());
                }
                Command::StartNewGame => {
                    self.scene = Scene::GameScene(GameScene::new());
                }
                Command::Exit => {
                    ggez::event::quit(&mut self.context.ggez_ctx);
                }
            }
        }
        Ok(())
    }
}

enum Scene {
    Intro(Intro),
    MainMenu(MainMenu),
    GameScene(GameScene),
}

struct Intro {
    remaining: Duration,
}

impl Intro {
    fn new() -> Intro {
        Intro {
            remaining: Duration::from_secs(1),
        }
    }
}

trait SceneEventHandler {
    fn update(&mut self, ctx: &mut Context) -> GameResult;
    fn draw(&mut self, ctx: &mut Context) -> GameResult;
    fn on_input(&mut self, ctx: &mut Context, button: &Button, state: &ButtonState) -> GameResult;
}

impl SceneEventHandler for Intro {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let delta = ctx.delta_time();
        if self.remaining > delta {
            self.remaining -= delta;
        } else {
            ctx.queue(Command::GoToMainMenu);
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        Ok(())
    }
    fn on_input(&mut self, ctx: &mut Context, button: &Button, state: &ButtonState) -> GameResult {
        Ok(())
    }
}

struct MainMenu {
    world: World,
}

impl MainMenu {
    fn new() -> MainMenu {
        let mut m = MainMenu {
            world: World::default(),
        };
        m.init();
        m
    }
    fn init(&mut self) {
        let font = Font::LiberationMono;
        self.world.push((
            Text::new("Lost and Found", font, 48),
            Position(Vec2::new(10.0, 20.0)),
        ));

        self.world.push((
            Text::new("Press space to start!", font, 40),
            Position(Vec2::new(10.0, 80.0)),
        ));

        self.world.push((
            Text::new("Or press esc to exit", font, 40),
            Position(Vec2::new(10.0, 140.0)),
        ));
    }
}

impl SceneEventHandler for MainMenu {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let res = Resources::default();

        let mut query = <(&Position, &Text)>::query();
        for (position, text) in query.iter_mut(&mut self.world) {
            ctx.draw_text(text, position)?;
        }

        Ok(())
    }
    fn on_input(&mut self, ctx: &mut Context, button: &Button, state: &ButtonState) -> GameResult {
        match button {
            Button::A => ctx.queue(Command::StartNewGame),
            Button::Start => ctx.queue(Command::Exit),
            _ => (),
        }
        Ok(())
    }
}

struct GameScene {}

impl GameScene {
    fn new() -> GameScene {
        GameScene {}
    }
}

impl SceneEventHandler for GameScene {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        Ok(())
    }
    fn on_input(&mut self, ctx: &mut Context, button: &Button, state: &ButtonState) -> GameResult {
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
    Game::new(cb.build()?)?.run()
}
