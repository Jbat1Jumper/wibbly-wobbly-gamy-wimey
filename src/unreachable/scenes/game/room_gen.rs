pub mod model {
    use crate::common::*;

    pub type DoorNumber = usize;
    pub type Room = &'static str;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Tile {
        Empty,
        Ground,
        Wall,
        Door(DoorNumber),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Object {
        Rock,
        Potion,
        Spikes,
    }

    pub struct RoomBlueprint {
        pub tiles: Vec<Vec<Tile>>,
        pub objects: Vec<Vec<Option<Object>>>,
        pub size: (usize, usize),
    }

    impl RoomBlueprint {
        pub fn tile_at(&self, pos: (i32, i32)) -> Tile {
            if self.in_bounds(pos) {
                self.tiles[pos.1 as usize][pos.0 as usize]
            } else {
                Tile::Empty
            }
        }

        #[rustfmt::skip]
        pub fn tile_neighborhood(&self, pos: (i32, i32)) -> [Tile; 9] {
            [
                self.tile_at(pos.step(Left).step(Up)),   self.tile_at(pos.step(Up)),   self.tile_at(pos.step(Right).step(Up)),
                self.tile_at(pos.step(Left)),            self.tile_at(pos),            self.tile_at(pos.step(Right)),
                self.tile_at(pos.step(Left).step(Down)), self.tile_at(pos.step(Down)), self.tile_at(pos.step(Right).step(Down)),
            ]
        }

        pub fn object_at(&self, pos: (i32, i32)) -> Option<Object> {
            if self.in_bounds(pos) {
                self.objects[pos.1 as usize][pos.0 as usize]
            } else {
                None
            }
        }

        pub fn in_bounds(&self, pos: (i32, i32)) -> bool {
            pos.0 >= 0 && pos.0 < self.size.0 as i32 && pos.1 >= 0 && pos.1 < self.size.1 as i32
        }

        pub fn positions(&self) -> Vec<(i32, i32)> {
            (0..self.size.1)
                .map(|y| (0..self.size.0).map(move |x| (x as i32, y as i32)))
                .flatten()
                .collect()
        }

    }

    pub trait RoomGenerator {
        //type Rng;
        fn create(room: Room /*, rng: &mut Self::Rng*/) -> RoomBlueprint;
    }
}

use model::*;

pub struct Lvl1RoomGenerator;

impl RoomGenerator for Lvl1RoomGenerator {
    fn create(room: Room) -> RoomBlueprint {
        use Tile::*;
        match room {
            #[rustfmt::skip]
            "S" => RoomBlueprint {
                size: (4, 5),
                tiles: vec![
                    vec![Wall, Wall,   Wall,   Wall],
                    vec![Wall, Ground, Ground, Wall],
                    vec![Wall, Ground, Ground, Door(1)],
                    vec![Wall, Ground, Ground, Wall],
                    vec![Wall, Wall,   Wall,   Wall],
                ],
                objects: vec![
                    vec![None, None, None, None],
                    vec![None, None, None, None],
                    vec![None, None, None, None],
                    vec![None, None, None, None],
                    vec![None, None, None, None],
                ],
            },
            r => panic!("Dont know how to create room {} at Lvl1", r),
        }
    }
}
