pub mod model {
    use crate::common::*;
    use crate::unreachable::scenes::game::level_gen::{Room, DoorNumber};

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

    #[derive(Clone, Debug)]
    pub struct RoomBlueprint {
        pub tiles: Vec<Tile>,
        pub objects: Option<Vec<Option<Object>>>,
        pub size: (usize, usize),
    }

    impl RoomBlueprint {
        pub fn tile_at(&self, pos: (i32, i32)) -> Tile {
            if self.in_bounds(pos) {
                self.tiles[(pos.0 as usize + pos.1 as usize * self.size.0)]
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
            if let Some(objects) = &self.objects {
                if self.in_bounds(pos) {
                    return objects[(pos.0 as usize + pos.1 as usize * self.size.0)]
                }
            }
            None
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
        fn create(&self, room: Room /*, rng: &mut Self::Rng*/) -> RoomBlueprint;
    }
}

