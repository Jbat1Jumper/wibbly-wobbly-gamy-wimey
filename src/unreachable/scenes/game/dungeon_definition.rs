use super::level_gen::*;
use super::room_gen::model::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct DungeonDefinition {
    pub mem_size: usize,
    pub start_room: Room,
    pub lvl_gen_rules: Vec<Rule>,
    pub rooms: HashMap<Room, RoomBlueprint>,
}

impl LevelGenDefinition for DungeonDefinition {
    fn mem_size(&self) -> usize {
        self.mem_size
    }
    fn start_room(&self) -> Room {
        self.start_room
    }
    fn get_rules(&self) -> Vec<Rule> {
        // Clone unnecesary
        self.lvl_gen_rules.clone()
    }
    fn available_doors(&self, r: Room) -> Vec<DoorNumber> {
        self.rooms
            .get(&r)
            .unwrap()
            .tiles
            .iter()
            .filter_map(|tile| match tile {
                Tile::Door(dn) => Some(*dn),
                _ => None,
            })
            .collect()
    }

    fn is_final(&self, r: Room) -> bool {
        r == "Final" || r == "F"
    }
}

impl RoomGenerator for DungeonDefinition {
    fn create(&self, room: Room) -> RoomBlueprint {
        self.rooms.get(room).unwrap().clone()
    }
}

#[rustfmt::skip]
pub fn lvl_1() -> DungeonDefinition {
    use Tile::*;
    DungeonDefinition {
        mem_size: 9,
        start_room: "S",
        lvl_gen_rules: vec![
            Rule::at("S")   .through(1)  .gets_to("a"),
            Rule::at("a")   .through(1)  .gets_to("b"),
            Rule::at("a")   .through(2)  .gets_to("c"),
            Rule::at("b")   .through(1)  .gets_to("d"),
            Rule::at("c")   .through(1)  .gets_to("d"),
            Rule::at("d")   .through(1)  .gets_to("F"),
        ],
        rooms: map!{
            "S" => RoomBlueprint {
                size: (5, 5),
                tiles: vec![
                    Wall, Wall,   Wall,   Wall,   Wall,
                    Wall, Ground, Ground, Ground, Wall,
                    Wall, Ground, Ground, Ground, Door(1),
                    Wall, Ground, Ground, Ground, Wall,
                    Wall, Wall,   Wall,   Wall,   Wall,
                ],
                objects: None,
            },
            "a" => RoomBlueprint {
                size: (5, 5),
                tiles: vec![
                    Wall,    Wall,   Door(1),   Wall,   Wall,
                    Wall,    Ground, Ground,    Ground, Wall,
                    Door(0), Ground, Ground,    Ground, Wall,
                    Wall,    Ground, Ground,    Ground, Wall,
                    Wall,    Wall,   Door(2),   Wall,   Wall,
                ],
                objects: None,
            },
            "b" => RoomBlueprint {
                size: (5, 5),
                tiles: vec![
                    Wall,  Wall,    Wall,     Wall,   Wall,
                    Wall,  Ground,  Ground,   Ground, Wall,
                    Wall,  Ground,  Ground,   Ground, Door(1),
                    Wall,  Ground,  Ground,   Ground, Wall,
                    Wall,  Wall,    Door(0),  Wall,   Wall,
                ],
                objects: None,
            },
            "c" => RoomBlueprint {
                size: (5, 5),
                tiles: vec![
                    Wall,  Wall,    Door(0),   Wall,   Wall,
                    Wall,  Ground,  Ground,    Ground, Wall,
                    Wall,  Ground,  Ground,    Ground, Door(1),
                    Wall,  Ground,  Ground,    Ground, Wall,
                    Wall,  Wall,    Wall,      Wall,   Wall,
                ],
                objects: None,
            },
            "d" => RoomBlueprint {
                size: (5, 5),
                tiles: vec![
                    Wall,    Wall,   Wall,   Wall,   Wall,
                    Wall,    Ground, Ground, Ground, Wall,
                    Door(0), Ground, Ground, Ground, Door(1),
                    Wall,    Ground, Ground, Ground, Wall,
                    Wall,    Wall,   Wall,   Wall,   Wall,
                ],
                objects: None,
            },
            "F" => RoomBlueprint {
                size: (5, 5),
                tiles: vec![
                    Wall,    Wall,   Wall,   Wall,   Wall,
                    Wall,    Ground, Ground, Ground, Wall,
                    Door(0), Ground, Ground, Ground, Wall,
                    Wall,    Ground, Ground, Ground, Wall,
                    Wall,    Wall,   Wall,   Wall,   Wall,
                ],
                objects: None,
            }
        },
    }
}
