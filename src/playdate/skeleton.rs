use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub type LinkId = usize;
pub type SlotName = char;

#[derive(Debug, Default, Eq, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SlotId(LinkId, SlotName);

pub type Variable = String;
pub type Effector = String;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Definition {
    pub last_link_id: LinkId,
    pub links: HashMap<LinkId, Link>,
    pub joints: Vec<(SlotId, Joint, SlotId)>,
    // variable_constrains: HashMap<String, VariableConstraint>,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Slot {
    pub position: Vec3,
    pub orientation: Quat,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// should be private
pub struct Link {
    pub slots: HashMap<SlotName, Slot>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Change {
    Add {
        link: Link,
        to_parent_slot: SlotId,
        joint: Joint,
        local_slot_name: SlotName,
    },
    ReplaceLink {
        at: LinkId,
        with: Link,
    },
    ReplaceJoint {
        at: SlotId,
        with: Joint,
    },
    Cut {
        from: SlotId,
    },
    MoveLink {
        at: SlotId,
        to: SlotId,
    },
    CutAndSplice {
        from: SlotId,
        to: SlotId,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Joint {
    Fixed,
    TwistingJoint(FloatValue),
    RotationalJoin(FloatValue),
    // LinearJoint(FloatValue),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FloatValue {
    Variable(Variable),
    Constant(f32),
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct VariableConstraint {
    pub min: Option<f32>,
    pub max: Option<f32>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ChangeError {
    UnexistingLink(LinkId),
    UnexistingSlot(SlotId),
    SlotOccupied(SlotId),
}

impl Definition {
    pub fn new(link0: Link) -> Self {
        Self {
            last_link_id: 0,
            links: hashmap! {
                0 => link0,
            },
            joints: vec![],
        }
    }

    pub fn apply(&mut self, change: Change) -> Result<(), ChangeError> {
        match change {
            Change::Add {
                link,
                to_parent_slot,
                joint,
                local_slot_name,
            } => {
                if !self.links.contains_key(&to_parent_slot.0) {
                    return Err(ChangeError::UnexistingLink(to_parent_slot.0));
                }
                if !self
                    .links
                    .get(&to_parent_slot.0)
                    .unwrap()
                    .slots
                    .contains_key(&to_parent_slot.1)
                {
                    return Err(ChangeError::UnexistingSlot(to_parent_slot));
                }
                if self
                    .joints
                    .iter()
                    .any(|(a, _j, b)| to_parent_slot == *a || to_parent_slot == *b)
                {
                    return Err(ChangeError::SlotOccupied(to_parent_slot));
                }

                self.last_link_id += 1;
                let id = self.last_link_id;
                self.links.insert(id, link);
                self.joints
                    .push((to_parent_slot, joint, SlotId(id, local_slot_name)));
            }
            _ => panic!(),
        }
        // Change::Delete(LinkId::Id(id)) => {
        //     self.entries.remove(&id);
        //     for (_, entry) in self.entries.iter_mut() {
        //         if entry.attached_to == Some(id) {
        //             entry.attached_to = None;
        //         }
        //     }
        // }
        // Change::AddToRoot(part) => {
        //     let id = self.last_link_id;
        //     self.last_link_id += 1;
        //     self.entries.insert(
        //         id,
        //         PartEntry {
        //             part,
        //             attached_to: None,
        //         },
        //     );
        // }
        // Change::AddToPart(pid, part) => {
        //     let id = self.last_link_id;
        //     self.last_link_id += 1;
        //     self.entries.insert(
        //         id,
        //         PartEntry {
        //             part,
        //             attached_to: Some(pid),
        //         },
        //     );
        // }
        // Change::Replace(id, part) => {
        //     self.entries
        //         .iter_mut()
        //         .find(|(eid, _)| **eid == id)
        //         .unwrap()
        //         .1
        //         .part = part.clone();
        // }
        // }
        Ok(())
    }

    pub fn variables(&self) -> HashSet<Variable> {
        todo!()
        // let mut set = HashSet::default();
        // for entry in self.entries.values() {
        //     match entry.part {
        //         Part::Rotation(FloatValue::Variable(Id var)) => {
        //             set.insert(var.clone());
        //         }
        //         _ => {}
        //     }
        // }
        // set
    }

    pub fn has_link(&self, link_id: LinkId) -> bool {
        self.links().contains(&link_id)
    }

    pub fn links(&self) -> Vec<LinkId> {
        let mut l: Vec<_> = self.links.keys().cloned().collect();
        l.sort();
        l
    }
    pub fn get_link_transform(&self, link_id: LinkId) -> Transform {
        self.get_link_transform_from(0, link_id)
    }
    fn get_link_transform_from(&self, origin_link_id: LinkId, target_link_id: LinkId) -> Transform {
        if origin_link_id == target_link_id {
            Transform::identity()
        } else {
            let translation_to_next = {
                let origin_link = self.links[&origin_link_id].clone();
                // Get shortest path P to target T from origin O
                // Get next link N in P
                // Get the pair of slots (with the joint) that connect the two links
                // Calculate the transform
                //   (now can just use the slot name to see if is 'l' or 'r')
                if true {
                    todo!("Implement what's above")
                } else {
                    Transform::from_translation(2.0 * UP)
                }
            };

            // TODO: change this implementation to be an iteration over the links in P instead of
            // a recursion
            translation_to_next * self.get_link_transform_from(origin_link_id + 1, target_link_id)
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Pose {
    pub valuation: HashMap<Variable, f32>,
}

#[macro_use]
use maplit::hashmap;

const LEFT: Vec3 = Vec3::X;
const FORWARD: Vec3 = Vec3::Y;
const UP: Vec3 = Vec3::Z;

mod link {
    use super::*;

    pub fn arm_base() -> Link {
        Link {
            slots: hashmap! {
                'n' => Slot {
                    position: 1.0 * UP ,
                    orientation: Quat::default(),
                },
            },
        }
    }

    pub fn l_link() -> Link {
        Link {
            slots: hashmap! {
                'p' => Slot {
                    position: -1.0 * UP,
                    orientation: Quat::default(),
                },
                'n' => Slot {
                    position: 1.0 * UP,
                    orientation: Quat::default(), // rotate
                },
            },
        }
    }

    pub fn t_link() -> Link {
        Link {
            slots: hashmap! {
                'p' => Slot {
                    position: -1.0 * UP,
                    orientation: Quat::default(),
                },
                'l' => Slot {
                    position: 1.0 * UP +1.0 * LEFT,
                    orientation: Quat::default(), // rotate
                },
                'r' => Slot {
                    position: 1.0 * UP -1.0 * LEFT,
                    orientation: Quat::default(), // rotate
                },
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::playdate::skeleton::*;

    #[test]
    fn assert_link0_transform_is_identity() {
        let s = Definition::new(link::arm_base());
        assert!(s.has_link(0));
        assert_eq!(s.links(), vec![0]);

        let t = s.get_link_transform(0);
        assert_eq!(t, Transform::identity())
    }

    #[test] #[ignore]
    fn can_add_link_with_fixed_joint() {
        let mut s = Definition::new(link::arm_base());
        s.apply(Change::Add {
            link: link::l_link(),
            to_parent_slot: SlotId(0, 'n'),
            joint: Joint::Fixed,
            local_slot_name: 'p',
        })
        .expect("Failed to add l_link");
        assert!(s.has_link(1));
        assert_eq!(s.links(), vec![0, 1]);

        let t = s.get_link_transform(1);
        assert_eq!(t, Transform::from_translation(2.0 * UP))
    }

    #[test]
    fn add_link_to_unexisting_slot_fails() {
        let mut s = Definition::new(link::arm_base());
        let res = s.apply(Change::Add {
            link: link::l_link(),
            to_parent_slot: SlotId(4, 'n'),
            joint: Joint::Fixed,
            local_slot_name: 'p',
        });
        assert_eq!(res, Err(ChangeError::UnexistingLink(4)));

        let res = s.apply(Change::Add {
            link: link::l_link(),
            to_parent_slot: SlotId(0, 'q'),
            joint: Joint::Fixed,
            local_slot_name: 'p',
        });
        assert_eq!(res, Err(ChangeError::UnexistingSlot(SlotId(0, 'q'))));
    }

    #[test] #[ignore]
    fn add_multiple_links_with_fixed_joint() {
        let mut s = Definition::new(link::arm_base());
        s.apply(Change::Add {
            link: link::l_link(),
            to_parent_slot: SlotId(0, 'n'),
            joint: Joint::Fixed,
            local_slot_name: 'p',
        })
        .expect("Failed to add first l_link");

        s.apply(Change::Add {
            link: link::l_link(),
            to_parent_slot: SlotId(1, 'n'),
            joint: Joint::Fixed,
            local_slot_name: 'p',
        })
        .expect("Failed to add second l_link");

        assert!(s.has_link(2));
        assert_eq!(s.links(), vec![0, 1, 2]);

        let t = s.get_link_transform(2);
        assert_eq!(t, Transform::from_translation(4.0 * UP));
    }

    #[test]
    fn add_two_links_to_the_same_slot_fails() {
        let mut s = Definition::new(link::arm_base());
        s.apply(Change::Add {
            link: link::l_link(),
            to_parent_slot: SlotId(0, 'n'),
            joint: Joint::Fixed,
            local_slot_name: 'p',
        })
        .expect("Failed to add first l_link");

        let res = s.apply(Change::Add {
            link: link::l_link(),
            to_parent_slot: SlotId(0, 'n'),
            joint: Joint::Fixed,
            local_slot_name: 'p',
        });
        assert_eq!(res, Err(ChangeError::SlotOccupied(SlotId(0, 'n'))));

        assert!(!s.has_link(2));
        assert_eq!(s.links(), vec![0, 1]);
    }

    #[test] #[ignore]
    fn add_t_link() {
        let mut s = Definition::new(link::arm_base());
        s.apply(Change::Add {
            link: link::t_link(),
            to_parent_slot: SlotId(0, 'n'),
            joint: Joint::Fixed,
            local_slot_name: 'p',
        })
        .expect("Failed to add t_link");
        s.apply(Change::Add {
            link: link::t_link(),
            to_parent_slot: SlotId(1, 'l'),
            joint: Joint::Fixed,
            local_slot_name: 'p',
        })
        .expect("Failed to add left l_link");
        s.apply(Change::Add {
            link: link::t_link(),
            to_parent_slot: SlotId(1, 'r'),
            joint: Joint::Fixed,
            local_slot_name: 'p',
        })
        .expect("Failed to add right l_link");

        assert_eq!(s.links(), vec![0, 1, 2, 3]);

        let t = s.get_link_transform(2);
        assert_eq!(t, Transform::from_translation(2.0 * UP + 2.0 * LEFT));

        let t = s.get_link_transform(3);
        assert_eq!(t, Transform::from_translation(2.0 * UP - 2.0 * LEFT));
    }

    // # TODO
    // ## Fixed Joint
    // - using_a_t_link_
    // ## Twisting Joint
    // - ...
    // ## Rotational Joint
    // - ...
    // - ...
    // ## Replace link and joint
    // ## Cut
    // ## Move link
    // -
}
