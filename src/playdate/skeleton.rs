use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub type LinkId = usize;
pub type SlotName = char;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
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
pub struct Link {
    pub slots: HashMap<SlotName, Slot>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Change {
    Replace {
        link: LinkId,
        with: Link,
    },
    Add {
        link: Link,
        to_parent_slot: SlotId,
        joint: Joint,
        local_slot_name: SlotName,
    },
    ReplaceJoint {
        at: SlotId,
        with: Joint,
    },
    Deatach {
        from: SlotId,
    },
    RemoveLink {
        at: SlotId,
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

#[derive(Debug, Clone)]
pub enum ChangeError {
    CannotRemoveLink0,
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

    pub fn has_link(&self, lr: LinkId) -> bool {
        true
    }

    pub fn links(&self) -> Vec<LinkId> {
        self.links.keys().cloned().collect()
    }
    pub fn get_link_frame_from(&self, origin: LinkId, target: LinkId) -> Transform {
        Transform::identity()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Pose {
    pub valuation: HashMap<Variable, f32>,
}

#[macro_use]
use maplit::hashmap;

// X points RIGHT
// Y points FORWARD
// Z points UP

fn arm_base_link() -> Link {
    Link {
        slots: hashmap! {
            'n' => Slot {
                position: Vec3::new(0.0, 0.0, 1.0),
                orientation: Quat::default(),
            },
        },
    }
}

fn l_link() -> Link {
    Link {
        slots: hashmap! {
            'p' => Slot {
                position: Vec3::new(0.0, 0.0, 1.0),
                orientation: Quat::default(),
            },
            'n' => Slot {
                position: Vec3::new(0.0, 0.0, -1.0),
                orientation: Quat::default(), // rotate
            },
        },
    }
}

#[cfg(test)]
mod test {
    use crate::playdate::skeleton::*;

    #[test]
    fn default_skeleton_has_only_a_link0() {
        let s = Definition::new(arm_base_link());
        assert!(s.has_link(0));
        assert_eq!(s.links(), vec![0]);
    }

    #[test]
    fn can_add_link_with_fixed_joint() {
        let mut s = Definition::new(arm_base_link());
        s.apply(Change::Add {
            link: l_link(),
            to_parent_slot: SlotId(0, 'n'),
            joint: Joint::Fixed,
            local_slot_name: 'p',
        })
        .expect("Failed to add l_link");
        assert!(s.has_link(1));
        assert_eq!(s.links(), vec![0, 1]);

        let t = s.get_link_frame_from(0, 1);

        assert_eq!(t, Transform::default())
    }
}
