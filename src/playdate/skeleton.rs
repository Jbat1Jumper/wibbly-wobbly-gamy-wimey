use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub type LinkRef = usize;
pub type SlotName = String;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct SlotRef(LinkRef, SlotName);

pub type Variable = String;
pub type Effector = String;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Definition {
    pub next_link_id: LinkRef,
    pub links: HashMap<LinkRef, Link>,
    pub joints: Vec<(SlotRef, Joint, SlotRef)>,
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
        link: LinkRef,
        with: Link,
    },
    Add {
        link: Link,
        to: SlotRef,
        with: Joint,
        on: SlotName,
    },
    ReplaceJoint {
        at: SlotRef,
        with: Joint,
    },
    Deatach {
        from: SlotRef,
    },
    RemoveLink {
        at: SlotRef,
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
pub struct UnexistingLink(LinkRef);

impl Definition {
    pub fn apply(&mut self, change: Change) -> Result<(), UnexistingLink> {
        // match change.clone() {
            // Change::Delete(LinkRef::Id(id)) => {
            //     self.entries.remove(&id);
            //     for (_, entry) in self.entries.iter_mut() {
            //         if entry.attached_to == Some(id) {
            //             entry.attached_to = None;
            //         }
            //     }
            // }
            // Change::AddToRoot(part) => {
            //     let id = self.next_link_id;
            //     self.next_link_id += 1;
            //     self.entries.insert(
            //         id,
            //         PartEntry {
            //             part,
            //             attached_to: None,
            //         },
            //     );
            // }
            // Change::AddToPart(pid, part) => {
            //     let id = self.next_link_id;
            //     self.next_link_id += 1;
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
        //         Part::Rotation(FloatValue::Variable(ref var)) => {
        //             set.insert(var.clone());
        //         }
        //         _ => {}
        //     }
        // }
        // set
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Pose {
    pub valuation: HashMap<Variable, f32>,
}

#[cfg(test)]
mod test {
    #[test]
    fn first_thing_first() {
        todo!("nooou")
    }
}
