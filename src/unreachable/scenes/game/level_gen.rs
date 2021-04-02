use regex::Regex;
use std::collections::HashMap;

#[rustfmt::skip]
type Room = &'static str;

type DoorNumber = usize;

type Age = usize;

#[derive(Clone, Debug)]
struct Rule(Regex, DoorNumber, Room);

struct IncompleteRule1(Regex);
struct IncompleteRule2(Regex, DoorNumber);

impl Rule {
    fn at(pattern: &'static str) -> IncompleteRule1 {
        IncompleteRule1(Regex::new(&format!("({})$", pattern)).unwrap())
    }
}

impl IncompleteRule1 {
    fn through(self, dn: DoorNumber) -> IncompleteRule2 {
        IncompleteRule2(self.0, dn)
    }
}

impl IncompleteRule2 {
    fn gets_to(self, room: Room) -> Rule {
        Rule(self.0, self.1, room)
    }
}

#[derive(Clone, Debug)]
struct RoomMemory {
    age: usize,
    connections: HashMap<DoorNumber, Room>,
}

#[derive(Clone, Debug)]
struct State {
    current_room: Room,
    memoized: HashMap<Room, RoomMemory>,
    visited: String,
    mem_size: usize,
}

#[derive(Clone, Debug)]
enum Problem {
    NoDoorInRoom(Room, DoorNumber),
    NoRuleMatchesFor(String, DoorNumber),
    MultipleMatchesFor(String, DoorNumber, Vec<Rule>),
    MultipleReachableMemoriesForRoom(String, usize),
}

type CreatedNewRoom = bool;

impl State {
    fn new(mem_size: usize, s: Room) -> Self {
        State {
            mem_size,
            current_room: s,
            visited: s.into(),
            memoized: map! {
                s => RoomMemory {
                    age: 0,
                    connections: HashMap::new(),
                }
            },
        }
    }

    fn step(&mut self, dn: DoorNumber) -> Result<CreatedNewRoom, Problem> {
        if !available_doors(self.current_room).contains(&dn) {
            return Err(Problem::NoDoorInRoom(self.current_room, dn));
        }

        let memory = self.memoized.get(&self.current_room).unwrap();

        // check the connection is memoized
        if let Some(next_room) = memory.connections.get(&dn).cloned() {
            // if so, just change current_room, push visited,
            //   increment all memory ages and set current_room's memory age to 0
            for (r, m) in self.memoized.iter_mut() {
                m.age = if *r == next_room { 0 } else { m.age + 1 };
            }
            self.current_room = next_room;
            self.visited.push_str(next_room);
            Ok(false)
        } else {
            // if not, then search for a rule to create a new room
            let visited = format!("_________{}", self.visited);
            let applicable_rules: Vec<_> = get_rules()
                .iter()
                .filter(|Rule(pattern, door, _)| *door == dn && pattern.is_match(&visited))
                .cloned()
                .collect();

            if applicable_rules.len() == 0 {
                return Err(Problem::NoRuleMatchesFor(self.visited.clone(), dn));
            }

            if applicable_rules.len() > 1 {
                return Err(Problem::MultipleMatchesFor(
                    self.visited.clone(),
                    dn,
                    applicable_rules,
                ));
            }

            let Rule(_, _, next_room) = applicable_rules.first().unwrap();

            // increment all memory ages by one
            for (r, m) in self.memoized.iter_mut() {
                m.age += 1;
            }

            let oldest_room = self
                .memoized
                .iter()
                .max_by_key(|(_, m)| m.age)
                .unwrap()
                .0
                .clone();

            // check if another memory of the current_room is reachable
            // (currently only if exists memory because now all memories
            // are reachable)
            if self.memoized.contains_key(next_room) {
                let is_going_to_be_removed =
                    *next_room == oldest_room && self.memoized.len() > self.mem_size;

                if !is_going_to_be_removed {
                    return Err(Problem::MultipleReachableMemoriesForRoom(
                        self.visited.clone(),
                        self.mem_size,
                    ));
                }
            }

            // add new memory (with age 0 and connections: 0 => self.current_room)
            self.memoized.insert(
                *next_room,
                RoomMemory {
                    age: 0,
                    connections: map! {
                        0 => self.current_room
                    },
                },
            );

            if self.memoized.len() > (self.mem_size + 1) && oldest_room != *next_room {
                self.memoized.remove(&oldest_room);
                for (r, m) in self.memoized.iter_mut() {
                    m.connections.retain(|dn, nr| *nr != oldest_room);
                }
            }

            self.current_room = *next_room;
            self.visited.push_str(next_room);
            Ok(true)
        }
    }
}

fn can_return(r: Room) -> bool {
    r != "S"
}

fn is_final(r: Room) -> bool {
    r == "F"
}

fn available_doors(r: Room) -> &'static [DoorNumber] {
    match r {
        "F" => &[0],
        "S" => &[1],
        "b" | "c" | "x" => &[0, 1],
        "a" => &[0, 1, 2],
        _ => panic!("Unknown room"),
    }
}

fn get_rules() -> [Rule; 8] {
    [
        Rule::at("S").through(1).gets_to("a"),
        Rule::at("a").through(1).gets_to("b"),
        Rule::at("a").through(2).gets_to("c"),
        Rule::at("[^c]ab|[^a]b").through(1).gets_to("x"),
        Rule::at("cab").through(1).gets_to("F"),
        Rule::at("[^b]ac|[^a]c").through(1).gets_to("x"),
        Rule::at("bac").through(1).gets_to("F"),
        Rule::at("x").through(1).gets_to("a"),
    ]
}

#[test]
pub fn test_trivial() {
    assert!(true, "Is ok");
}

#[test]
pub fn test_regex() {
    let p = "([^b]ac|[^a]c)$";
    assert!(
        Regex::new(p).unwrap().is_match("xxxxc"),
        "{} Should match xxxxc",
        p
    );
    assert!(
        Regex::new(p).unwrap().is_match("cac"),
        "{} Should match cac",
        p
    );
    assert!(
        !Regex::new(p).unwrap().is_match("Saca"),
        "{} Should not match Saca",
        p
    );
    assert!(
        !Regex::new(p).unwrap().is_match("bac"),
        "{} Should not match bac",
        p
    );
}

#[test]
pub fn can_do_single_step() {
    let mut state = State::new(0, "S");
    state.step(1).unwrap();
    assert_eq!(state.current_room, "a", "Current room differs");
}

#[test]
pub fn walk_till_X() {
    let mut state = State::new(0, "S");
    state.step(1).unwrap();
    state.step(2).unwrap();
    assert_eq!(state.current_room, "c", "Current room differs");
    state.step(1).unwrap();
    assert_eq!(state.current_room, "x", "Current room differs");
}

#[test]
pub fn walk_with_no_return() {
    let mut state = State::new(0, "S");
    state.step(1).unwrap();
    state.step(2).unwrap();
    assert_eq!(state.current_room, "c", "Current room differs");
    let r = state.step(0);
    if let Err(Problem::NoRuleMatchesFor(_, _)) = r {
    } else {
        panic!("Unexpected result: {:?}, at state: {:?}", r, state);
    }
}

#[test]
pub fn walk_with_return() {
    let mut state = State::new(1, "S");
    state.step(1).unwrap();
    state.step(2).unwrap();
    assert_eq!(state.current_room, "c", "Current room differs");
    state.step(0).unwrap();
    assert_eq!(state.current_room, "a", "Current room differs");
}

#[test]
pub fn walk_till_F() {
    let mut state = State::new(3, "S");
    state.step(1).unwrap();
    state.step(2).unwrap();
    state.step(0).unwrap();
    state.step(1).unwrap();
    assert_eq!(state.current_room, "b", "Current room differs");
    state.step(1).unwrap();
    assert_eq!(state.current_room, "F", "Current room differs");
}

#[test]
pub fn cant_have_two_a() {
    let mut state = State::new(3, "S");
    state.step(1).unwrap();
    state.step(1).unwrap();
    state.step(1).unwrap();
    let r = state.step(1);
    if let Err(Problem::MultipleReachableMemoriesForRoom(_, _)) = r {
    } else {
        panic!("Unexpected result: {:?}, at state: {:?}", r, state);
    }
}

#[test]
pub fn if_older_room_is_removed_is_ok_that_is_the_same() {
    let mut state = State::new(2, "S");
    state.step(1).unwrap();
    state.step(1).unwrap();
    state.step(1).unwrap();
    state.step(1).unwrap();
    assert_eq!(
        state.current_room, "a",
        "Current room should be 'a', at state: {:?}",
        state
    );
}

#[test]
pub fn should_fail_if_older_room_is_the_same_as_the_new_but_isnt_removed() {
    let mut state = State::new(3, "a");
    state.step(1).unwrap();
    state.step(1).unwrap();
    let r = state.step(1);
    if let Err(Problem::MultipleReachableMemoriesForRoom(_, _)) = r {
    } else {
        panic!("Unexpected result: {:?}, at state: {:?}", r, state);
    }
}
