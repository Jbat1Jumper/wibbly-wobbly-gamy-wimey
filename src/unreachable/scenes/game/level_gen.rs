use std::collections::{HashMap, HashSet};
use Room::*;

#[rustfmt::skip]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum Room { S, A, B, C, F, X, }

type DoorNumber = usize;

type Age = usize;

#[derive(Clone, Debug)]
struct Rule(&'static [Room], DoorNumber, Room);

#[derive(Clone, Debug)]
struct RoomMemory {
    age: usize,
    connections: HashMap<DoorNumber, Room>,
}

#[derive(Clone, Debug)]
struct State {
    current_room: Room,
    memoized: HashMap<Room, RoomMemory>,
    visited: Vec<Room>,
    mem_size: usize,
}

#[derive(Clone, Debug)]
enum Problem {
    NoDoorInRoom(Room, DoorNumber),
    NoRuleMatchesFor(Vec<Room>, DoorNumber),
    MultipleMatchesFor(Vec<Room>, DoorNumber, Vec<Rule>),
    MultipleReachableMemoriesForRoom(Vec<Room>, usize),
}

type CreatedNewRoom = bool;

impl State {
    fn new(mem_size: usize) -> Self {
        State {
            mem_size,
            current_room: S,
            visited: vec![S],
            memoized: map! {
                S => RoomMemory {
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
            self.visited.push(next_room);
            Ok(false)
        } else {
            // if not, then search for a rule to create a new room
            let applicable_rules: Vec<_> = get_rules()
                .iter()
                .filter(|Rule(pattern, door, _)| *door == dn && self.visited.ends_with(pattern))
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
            if self.memoized.contains_key(next_room) && *next_room != oldest_room {
                return Err(Problem::MultipleReachableMemoriesForRoom(
                    self.visited.clone(),
                    self.mem_size,
                ));
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

            if self.memoized.len() > (self.mem_size + 1) {
                self.memoized.remove(&oldest_room);
                for (r, m) in self.memoized.iter_mut() {
                    m.connections.retain(|dn, nr| *nr != oldest_room);
                }
            }

            self.current_room = *next_room;
            self.visited.push(*next_room);
            Ok(true)
        }
    }
}

fn can_return(r: Room) -> bool {
    r != S
}

fn is_final(r: Room) -> bool {
    r == F
}

fn available_doors(r: Room) -> &'static [DoorNumber] {
    match r {
        F => &[0],
        S => &[1],
        B | C | X => &[0, 1],
        A => &[0, 1, 2],
    }
}

fn get_rules() -> &'static [Rule] {
    &[
        Rule(&[S], 1, A),
        Rule(&[A], 1, B),
        Rule(&[A], 2, C),
        Rule(&[S, A, B], 1, X),
        Rule(&[B, A, B], 1, X),
        Rule(&[B, X, B], 1, X),
        Rule(&[X, A, B], 1, X),
        Rule(&[C, A, B], 1, F),
        Rule(&[S, A, C], 1, X),
        Rule(&[C, A, C], 1, X),
        Rule(&[C, X, C], 1, X),
        Rule(&[X, A, C], 1, X),
        Rule(&[B, A, C], 1, F),
        Rule(&[X], 1, A),
    ]
}

#[test]
pub fn test_trivial() {
    assert!(true, "Is ok");
}

#[test]
pub fn can_do_single_step() {
    let mut state = State::new(0);
    state.step(1).unwrap();
    assert_eq!(state.current_room, A, "Current room differs");
}

#[test]
pub fn walk_till_X() {
    let mut state = State::new(0);
    state.step(1).unwrap();
    state.step(2).unwrap();
    assert_eq!(state.current_room, C, "Current room differs");
    state.step(1).unwrap();
    assert_eq!(state.current_room, X, "Current room differs");
}

#[test]
pub fn walk_with_no_return() {
    let mut state = State::new(0);
    state.step(1).unwrap();
    state.step(2).unwrap();
    assert_eq!(state.current_room, C, "Current room differs");
    let r = state.step(0);
    if let Err(Problem::NoRuleMatchesFor(_, _)) = r {
    } else {
        panic!("Unexpected result: {:?}, at state: {:?}", r, state);
    }
}

#[test]
pub fn walk_with_return() {
    let mut state = State::new(1);
    state.step(1).unwrap();
    state.step(2).unwrap();
    assert_eq!(state.current_room, C, "Current room differs");
    state.step(0).unwrap();
    assert_eq!(state.current_room, A, "Current room differs");
}

#[test]
pub fn walk_till_F() {
    let mut state = State::new(3);
    state.step(1).unwrap();
    state.step(2).unwrap();
    state.step(0).unwrap();
    state.step(1).unwrap();
    assert_eq!(state.current_room, B, "Current room differs");
    state.step(1).unwrap();
    assert_eq!(state.current_room, F, "Current room differs");
}
