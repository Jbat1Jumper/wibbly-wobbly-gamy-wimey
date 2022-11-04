#[macro_use]
pub use maplit::hashmap;
use std::{
    collections::HashMap,
    fmt::{Display, Write},
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ArtifactReference(String);
pub fn ar<T: Into<String>>(x: T) -> ArtifactReference {
    ArtifactReference(x.into())
}

impl Display for ArtifactReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //f.write_str("@")?;
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SlotName(String);
pub fn sn<T: Into<String>>(x: T) -> SlotName {
    SlotName(x.into())
}
impl Display for SlotName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("'")?;
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SlotKind(String);
pub fn sk<T: Into<String>>(x: T) -> SlotKind {
    SlotKind(x.into())
}

impl Display for SlotKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        self.0.fmt(f)?;
        f.write_str("]")
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Block {
    pub main_slot_kind: SlotKind,
    pub slots: HashMap<SlotName, SlotKind>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Artifact {
    Block(Block),
    Structure(Structure),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Structure {
    pub a_ref: ArtifactReference,
    pub c: HashMap<SlotName, Connection>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Connection {
    Structure(Structure),
    Slot(SlotName),
}

#[derive(Debug)]
pub enum ModelValidationError {
    Artifact(ArtifactReference, ArtifactValidationError),
}

#[derive(Debug)]
pub enum ArtifactValidationError {
    GettingKindOf(GettingKindOfError),
    GettingSlotOf(GettingSlotOfError),
}

#[derive(Debug)]
pub enum GettingKindOfError {
    Unexistent(Vec<ArtifactReference>, ArtifactReference),
    RecursionDetected(Vec<ArtifactReference>, ArtifactReference),
}
#[derive(Debug)]
pub struct GettingSlotOfError(String);

// TODO: Indicate which methods are safe to use (do not invalidate model), and which ones can panic or
// overflow if the model is not valid.
pub trait Model: Send + Sync {
    fn set_artifact(&mut self, aref: ArtifactReference, artifact: Artifact);
    fn remove_artifact(&mut self, aref: &ArtifactReference);
    fn get_artifact(&self, aref: &ArtifactReference) -> Option<&Artifact>;

    // TODO: This could be an iterator to easily support infinite models...
    fn list_artifacts(&self) -> Vec<ArtifactReference>;
    // ... and expose something like this to hint users of what to expect of this.
    fn is_finite(&self) -> bool {
        true
    }

    fn exists_artifact(&self, aref: &ArtifactReference) -> bool {
        self.get_artifact(aref).is_some()
    }

    fn is_all_valid(&self) -> bool {
        self.validate_model().is_ok()
    }

    fn validate_model(&self) -> Result<(), ModelValidationError> {
        for aref in self.list_artifacts().into_iter() {
            self.validate(&aref)
                .map_err(|err| ModelValidationError::Artifact(aref, err))?;
        }
        Ok(())
    }

    fn validate(&self, aref: &ArtifactReference) -> Result<(), ArtifactValidationError> {
        self.main_slot_kind_of(aref)
            .map_err(ArtifactValidationError::GettingKindOf)?;
        self.slots_of(aref)
            .map_err(ArtifactValidationError::GettingSlotOf)?;
        Ok(())
    }

    fn safely_remove_block_slot(
        &mut self,
        aref: &ArtifactReference,
        slot_name: &SlotName,
    ) -> Result<(), String> {
        if !self.is_all_valid() {
            return Err("Cannot safely remove slot in an invalid model".into());
        }

        match self.get_artifact(aref) {
            Some(Artifact::Block(block)) => {
                let mut deps: Vec<_> = self
                    .direct_dependents(aref)
                    .into_iter()
                    .filter(|dref| {
                        if let Artifact::Structure(structure) = self.get_artifact(dref).unwrap() {
                            self.structure_references_artifact_slot(structure, aref, slot_name)
                        } else {
                            unreachable!("A block cannot be a direct dependant of aref")
                        }
                    })
                    .collect();
                if deps.is_empty() {
                    let mut block = block.clone(); // TODO: This unnecesary memory allocations could go away with a `self.get_artifact_mut`.
                    match block.slots.remove(slot_name) {
                        Some(_slot_kind) => {
                            self.set_artifact(aref.clone(), Artifact::Block(block));
                            Ok(())
                        }
                        None => Err(format!("Block {} does not have a {} slot", aref, slot_name)),
                    }
                } else {
                    // TODO: This is probably implemented in some lib. If not, is worth moving to a
                    // prelude as it will be probably used in many places.
                    let references: String = if deps.len() == 1 {
                        deps.first().unwrap().0.clone()
                    } else {
                        deps.sort_by(|a, b| a.0.cmp(&b.0));
                        let all_but_last = deps
                            .iter()
                            .rev()
                            .skip(1)
                            .rev()
                            .cloned()
                            .map(|dep_aref| dep_aref.0)
                            .collect::<Vec<_>>()
                            .join(", ");
                        let last = deps.last().unwrap().0.clone();
                        format!("{} and {}", all_but_last, last)
                    };
                    Err(format!(
                        "Cannot remove slot {} from {} because is used by {}",
                        slot_name, aref, references
                    ))
                }
            }
            None => Err(format!("Artifact {} does not exist", aref)),
            Some(Artifact::Structure(_)) => Err("Cannot remove slot of a structure".into()),
        }
    }

    fn structure_references_artifact_slot(
        &self,
        structure: &Structure,
        referenced_slot_aref: &ArtifactReference,
        referenced_slot_name: &SlotName,
    ) -> bool {
        if structure.a_ref == *referenced_slot_aref
            && structure.c.contains_key(referenced_slot_name)
        {
            true
        } else {
            let mut substructs = structure.c.iter().filter_map(|(_, c)| {
                if let Connection::Structure(s) = c {
                    Some(s)
                } else {
                    None
                }
            });
            substructs.any(|substruct| {
                self.structure_references_artifact_slot(
                    substruct,
                    referenced_slot_aref,
                    referenced_slot_name,
                )
            })
        }
    }

    fn safely_remove_artifact(&mut self, aref: &ArtifactReference) -> Result<(), String> {
        if self.is_all_valid() {
            let mut deps = self.direct_dependents(aref);
            if deps.is_empty() {
                self.remove_artifact(&aref);
                Ok(())
            } else {
                // TODO: This is probably implemented in some lib. If not, is worth moving to a
                // prelude as it will be probably used in many places.
                let references: String = if deps.len() == 1 {
                    deps.first().unwrap().0.clone()
                } else {
                    deps.sort_by(|a, b| a.0.cmp(&b.0));
                    let all_but_last = deps
                        .iter()
                        .rev()
                        .skip(1)
                        .rev()
                        .cloned()
                        .map(|dep_aref| dep_aref.0)
                        .collect::<Vec<_>>()
                        .join(", ");
                    let last = deps.last().unwrap().0.clone();
                    format!("{} and {}", all_but_last, last)
                };
                Err(format!(
                    "Cannot safely remove {} because is referenced by {}",
                    aref, references
                ))
            }
        } else {
            Err("Cannot safely remove an artifact from an invalid model".into())
        }
    }

    fn rename_slot(
        &mut self,
        aref: &ArtifactReference,
        old_slot_name: &SlotName,
        new_slot_name: SlotName,
    ) -> Result<(), String> {
        match self
            .get_artifact(aref)
            .ok_or_else(|| format!("Artifact {} does not exists ", aref))?
        {
            Artifact::Block(ref block) => {
                let mut block = block.clone();
                if let Some(slot_kind) = block.slots.get(old_slot_name).cloned() {
                    if !block.slots.contains_key(&new_slot_name) {
                        block.slots.remove(old_slot_name);
                        block.slots.insert(new_slot_name.clone(), slot_kind);
                        self.set_artifact(aref.clone(), Artifact::Block(block));
                        Ok(())
                    } else {
                        Err(format!(
                            "Artifact {} already has a slot named {}",
                            aref, new_slot_name
                        ))
                    }
                } else {
                    Err(format!(
                        "Slot {} does not exists in {}",
                        old_slot_name, aref
                    ))
                }
            }
            Artifact::Structure(_structure) => todo!(),
        }
        .and_then(|_| {
            for dref in self.direct_dependents(aref) {
                if let Some(Artifact::Structure(mut structure)) = self.get_artifact(&dref).cloned()
                {
                    self.rename_slot_in_structure(
                        &mut structure,
                        aref,
                        old_slot_name,
                        new_slot_name.clone(),
                    );
                    self.set_artifact(dref, Artifact::Structure(structure));
                } else {
                    unreachable!()
                }
            }
            Ok(())
        })
    }

    fn rename_slot_in_structure(
        &mut self,
        structure: &mut Structure,
        aref: &ArtifactReference,
        old_slot_name: &SlotName,
        new_slot_name: SlotName,
    ) {
        if structure.a_ref == *aref {
            let connection = structure.disconnect(Location(vec![]), old_slot_name);
            structure.connect(Location(vec![]), &new_slot_name, connection);
        }

        for (_slot_name, connection) in structure.c.iter_mut() {
            if let Connection::Structure(ref mut substruct) = connection {
                self.rename_slot_in_structure(
                    substruct,
                    aref,
                    old_slot_name,
                    new_slot_name.clone(),
                );
            }
        }
    }

    fn main_slot_kind_of(&self, aref: &ArtifactReference) -> Result<SlotKind, GettingKindOfError> {
        self.main_slot_kind_of_with_breadcrumb(aref, vec![])
    }
    fn main_slot_kind_of_with_breadcrumb(
        &self,
        aref: &ArtifactReference,
        mut breadcrumb: Vec<ArtifactReference>,
    ) -> Result<SlotKind, GettingKindOfError> {
        match self
            .get_artifact(aref)
            .ok_or_else(|| GettingKindOfError::Unexistent(breadcrumb.clone(), aref.clone()))?
        {
            Artifact::Block(ref block) => Ok(block.main_slot_kind.clone()),
            Artifact::Structure(structure) => {
                breadcrumb.push(aref.clone());
                if breadcrumb.contains(&structure.a_ref) {
                    return Err(GettingKindOfError::RecursionDetected(
                        breadcrumb.clone(),
                        structure.a_ref.clone(),
                    ));
                }
                self.main_slot_kind_of_with_breadcrumb(&structure.a_ref, breadcrumb)
            }
        }
    }

    fn slots_of(
        &self,
        aref: &ArtifactReference,
    ) -> Result<HashMap<SlotName, SlotKind>, GettingSlotOfError> {
        match self.get_artifact(aref).ok_or(GettingSlotOfError(format!(
            "Could not get artifact {}",
            aref
        )))? {
            Artifact::Block(ref block) => Ok(block.slots.clone()),
            Artifact::Structure(structure) => {
                if *aref == structure.a_ref {
                    return Err(GettingSlotOfError(format!(
                        "Structure {} references to itself",
                        aref
                    )));
                }
                self.slots_of_structure(structure)
            }
        }
    }
    fn slots_of_structure(
        &self,
        structure: &Structure,
    ) -> Result<HashMap<SlotName, SlotKind>, GettingSlotOfError> {
        let inner_slots = self.slots_of(&structure.a_ref)?;

        let mut slots = hashmap! {};

        for (slot_name, slot_kind) in inner_slots {
            match structure.c.get(&slot_name) {
                Some(c) => match c {
                    Connection::Slot(outer_slot_name) => {
                        slots.insert(outer_slot_name.clone(), slot_kind.clone());
                    }
                    Connection::Structure(child_structure) => {
                        for (slot_name, slot_kind) in self.slots_of_structure(child_structure)? {
                            slots.insert(slot_name, slot_kind);
                        }
                    }
                },
                None => {
                    // This is not an error, is just not connected.
                    // return Err(GettingSlotOfError(format!(
                    //     "Slot {} is not connected to anything",
                    //     slot_name
                    // )))
                }
            }
        }

        Ok(slots)
    }

    fn direct_dependencies(&self, aref: &ArtifactReference) -> Vec<ArtifactReference> {
        match self.get_artifact(aref).unwrap() {
            Artifact::Structure(s) => {
                let mut structures_to_see = vec![s];
                let mut ds = vec![];
                while !structures_to_see.is_empty() {
                    let s = structures_to_see.pop().unwrap();
                    ds.push(s.a_ref.clone());
                    for c in s.c.values() {
                        match c {
                            Connection::Structure(s2) => structures_to_see.push(s2),
                            _ => (),
                        }
                    }
                }
                ds
            }
            Artifact::Block(_) => vec![],
        }
    }

    fn dependencies(&self, aref: &ArtifactReference) -> Vec<ArtifactReference> {
        let mut ds = self.direct_dependencies(aref);
        let tds: Vec<_> = ds
            .iter()
            .map(|aref2| self.dependencies(aref2))
            .flatten()
            .collect();
        ds.extend(tds);
        ds.dedup();
        ds
    }

    fn direct_dependents(&self, aref: &ArtifactReference) -> Vec<ArtifactReference> {
        self.list_artifacts()
            .into_iter()
            .filter(|other_aref| self.direct_dependencies(&other_aref).contains(aref))
            .collect()
    }

    fn dependents(&self, aref: &ArtifactReference) -> Vec<ArtifactReference> {
        let mut ds = self.direct_dependents(aref);
        let tds: Vec<_> = ds
            .iter()
            .map(|aref2| self.dependents(aref2))
            .flatten()
            .collect();
        ds.extend(tds);
        ds.dedup();
        ds
    }

    fn union(&self, other_model: &dyn Model) -> InMemoryModel {
        todo!()
    }
}

pub struct Location(Vec<SlotName>);

impl Artifact {
    fn composite(&self) -> bool {
        if let Artifact::Structure(_) = self {
            true
        } else {
            false
        }
    }
}

impl Structure {
    fn swap(&mut self, target: Location, new_aref: ArtifactReference) {
        todo!();
    }

    fn disconnect(&mut self, target: Location, slot_name: &SlotName) -> Connection {
        self.c.remove(slot_name).unwrap()
    }

    fn connect(&mut self, target: Location, slot_name: &SlotName, connection: Connection) {
        self.c.insert(slot_name.clone(), connection);
    }
}

#[derive(Debug, Default)]
pub struct InMemoryModel(HashMap<ArtifactReference, Artifact>);

impl Model for InMemoryModel {
    fn set_artifact(&mut self, aref: ArtifactReference, artifact: Artifact) {
        self.0.insert(aref, artifact);
    }

    fn remove_artifact(&mut self, aref: &ArtifactReference) {
        self.0.remove(aref);
    }

    fn get_artifact(&self, aref: &ArtifactReference) -> Option<&Artifact> {
        self.0.get(aref)
    }

    fn list_artifacts(&self) -> Vec<ArtifactReference> {
        self.0.keys().cloned().collect()
    }
}

pub mod example {
    use super::*;

    pub fn empty_test_model() -> InMemoryModel {
        InMemoryModel(hashmap! {})
    }

    pub fn peano_model() -> InMemoryModel {
        let mut model = empty_test_model();
        model.set_artifact(
            ar("successor"),
            Artifact::Block(Block {
                main_slot_kind: sk("Natural"),
                slots: hashmap! {
                    sn("x") => sk("Natural"),
                },
            }),
        );
        model.set_artifact(
            ar("zero"),
            Artifact::Block(Block {
                main_slot_kind: sk("Natural"),
                slots: hashmap! {},
            }),
        );
        model
    }

    pub fn peano_eval<M: Model>(aref: &ArtifactReference, model: &M) -> usize {
        if *aref == ar("number_2") {
            2
        } else if *aref == ar("number_4") {
            4
        } else {
            todo!("Not implemented")
        }
    }

    pub fn two_and_two_is_four_model() -> InMemoryModel {
        let mut model = peano_model();
        model.set_artifact(
            ar("plus_2"),
            Artifact::Structure(Structure {
                a_ref: ar("successor"),
                c: hashmap! {
                    sn("x") => Connection::Structure(Structure {
                        a_ref: ar("successor"),
                        c: hashmap!{
                            sn("x") => Connection::Slot(sn("x"))
                        }
                    })
                },
            }),
        );
        model.set_artifact(
            ar("number_4"),
            Artifact::Structure(Structure {
                a_ref: ar("plus_2"),
                c: hashmap! {
                    sn("x") => Connection::Structure(Structure {
                        a_ref: ar("plus_2"),
                        c: hashmap!{
                            sn("x") => Connection::Structure(Structure {
                                a_ref: ar("zero"),
                                c: hashmap!{}
                            })
                        }
                    })
                },
            }),
        );
        model
    }

    #[test]
    fn peano_numbers() {
        let mut model = peano_model();
        model.set_artifact(
            ar("number_2"),
            Artifact::Structure(Structure {
                a_ref: ar("successor"),
                c: hashmap! {
                    sn("x") => Connection::Structure(Structure {
                        a_ref: ar("successor"),
                        c: hashmap!{
                            sn("x") => Connection::Structure(Structure {
                                a_ref: ar("zero"),
                                c: hashmap!{}
                            })
                        }
                    })
                },
            }),
        );
        model.validate_model().unwrap();
        assert!(model.slots_of(&ar("number_2")).unwrap().is_empty());
        assert_eq!(peano_eval(&ar("number_2"), &model), 2);
    }

    pub fn and_one_more_is_five_model() -> InMemoryModel {
        let mut model = two_and_two_is_four_model();
        model.set_artifact(
            ar("number_5"),
            Artifact::Structure(Structure {
                a_ref: ar("successor"),
                c: hashmap! {
                    sn("x") => Connection::Structure(Structure {
                        a_ref: ar("number_4"),
                        c: hashmap!{ },
                    })
                },
            }),
        );
        model
    }
}
#[cfg(test)]
mod test {
    use super::example::*;
    use super::*;

    #[test]
    fn empty_model_is_valid() {
        let mut model = empty_test_model();
        model.validate_model().unwrap();
        assert!(model.list_artifacts().is_empty());
    }

    #[test]
    fn a_simple_block() {
        // Given
        let mut model = empty_test_model();
        let a = ar("a");
        model.set_artifact(
            a.clone(),
            Artifact::Block(Block {
                main_slot_kind: sk("A"),
                slots: hashmap! {},
            }),
        );
        model.validate_model().unwrap();
        assert_eq!(model.main_slot_kind_of(&a).unwrap(), sk("A"));
        assert_eq!(model.slots_of(&a).unwrap(), hashmap! {});

        let artifact = model.get_artifact(&a).unwrap();
        assert_eq!(artifact.composite(), false);

        assert!(model.list_artifacts().contains(&a));
    }

    #[test]
    fn a_block_with_a_slot() {
        let mut model = empty_test_model();
        model.set_artifact(
            ar("a"),
            Artifact::Block(Block {
                main_slot_kind: sk("A"),
                slots: hashmap! {
                    sn("1") => sk("A")
                },
            }),
        );
        model.validate_model().unwrap();
        assert!(model.is_all_valid());
        assert_eq!(model.main_slot_kind_of(&ar("a")).unwrap(), sk("A"));
        assert_eq!(model.slots_of(&ar("a")).unwrap()[&sn("1")], sk("A"));

        let artifact = model.get_artifact(&ar("a")).unwrap();
        assert_eq!(artifact.composite(), false);

        assert!(model.list_artifacts().contains(&ar("a")));
    }

    #[test]
    fn a_structure_depending_on_an_unexistent_artifact() {
        let mut model = empty_test_model();
        model.set_artifact(
            ar("a"),
            Artifact::Structure(Structure {
                a_ref: ar("b"),
                c: hashmap! {},
            }),
        );
        model.validate_model().expect_err("Should fail");
        assert!(!model.is_all_valid());
        assert!(model.list_artifacts().contains(&ar("a")));
        assert!(model.get_artifact(&ar("a")).unwrap().composite());
    }

    #[test]
    fn a_structure_using_a_simple_block() {
        let mut model = empty_test_model();
        model.set_artifact(
            ar("b"),
            Artifact::Block(Block {
                main_slot_kind: sk("B"),
                slots: hashmap! {},
            }),
        );
        model.set_artifact(
            ar("a"),
            Artifact::Structure(Structure {
                a_ref: ar("b"),
                c: hashmap! {},
            }),
        );
        model.validate_model().unwrap();

        assert_eq!(model.main_slot_kind_of(&ar("a")).unwrap(), sk("B"));
        assert_eq!(model.slots_of(&ar("a")).unwrap().is_empty(), true);
    }

    #[test]
    fn a_structure_using_a_simple_block_remapping_slots() {
        let mut model = empty_test_model();
        model.set_artifact(
            ar("b"),
            Artifact::Block(Block {
                main_slot_kind: sk("B"),
                slots: hashmap! {
                    sn("80") => sk("PROTO")
                },
            }),
        );
        model.set_artifact(
            ar("a"),
            Artifact::Structure(Structure {
                a_ref: ar("b"),
                c: hashmap! {
                    sn("80") => Connection::Slot(sn("9980")),
                },
            }),
        );
        model.validate_model().unwrap();

        assert_eq!(model.main_slot_kind_of(&ar("a")).unwrap(), sk("B"));
        assert_eq!(
            model.slots_of(&ar("a")).unwrap(),
            hashmap! {sn("9980") => sk("PROTO")}
        );
    }

    #[test]
    fn a_structure_depending_on_itself() {
        let mut model = empty_test_model();
        model.set_artifact(
            ar("a"),
            Artifact::Structure(Structure {
                a_ref: ar("a"),
                c: hashmap! {},
            }),
        );
        model.validate_model().expect_err("Should fail");
    }

    #[test]
    fn mutually_recursive_structures() {
        let mut model = empty_test_model();
        model.set_artifact(
            ar("a"),
            Artifact::Structure(Structure {
                a_ref: ar("b"),
                c: hashmap! {},
            }),
        );
        model.set_artifact(
            ar("b"),
            Artifact::Structure(Structure {
                a_ref: ar("a"),
                c: hashmap! {},
            }),
        );
        model.validate_model().expect_err("Should fail");
    }

    #[test]
    fn two_and_two_is_four() {
        let mut model = two_and_two_is_four_model();
        model.validate_model().unwrap();
        assert_eq!(model.slots_of(&ar("plus_2")).unwrap().keys().count(), 1);
        assert_eq!(
            model.slots_of(&ar("plus_2")).unwrap()[&sn("x")],
            sk("Natural")
        );
        assert!(model.slots_of(&ar("number_4")).unwrap().is_empty());

        assert_eq!(peano_eval(&ar("number_4"), &model), 4);
    }

    #[test]
    fn cannot_safely_remove_an_artifact_from_an_invalid_model() {
        let mut model = two_and_two_is_four_model();
        model.remove_artifact(&ar("zero"));

        let res = model.safely_remove_artifact(&ar("number_4"));
        assert_eq!(
            res,
            Err("Cannot safely remove an artifact from an invalid model".into())
        );
        assert!(model.list_artifacts().contains(&ar("number_4")));
    }

    #[test]
    fn can_safely_remove_something_that_is_not_referenced_anywhere() {
        let mut model = two_and_two_is_four_model();
        let res = model.safely_remove_artifact(&ar("number_4"));
        assert_eq!(res, Ok(()));
        model.validate_model().unwrap();
        assert!(!model.list_artifacts().contains(&ar("number_4")));
    }

    #[test]
    fn cannot_remove_an_artifact_that_is_referenced_by_a_structure() {
        let mut model = two_and_two_is_four_model();
        let res = model.safely_remove_artifact(&ar("zero"));
        assert_eq!(
            res,
            Err("Cannot safely remove zero because is referenced by number_4".into())
        );
        model.validate_model().unwrap();
        assert!(model.list_artifacts().contains(&ar("zero")));
        assert!(model.list_artifacts().contains(&ar("number_4")));
    }

    #[test]
    fn cannot_remove_another_artifact_that_is_referenced_by_multiple_structures() {
        let mut model = and_one_more_is_five_model();
        let res = model.safely_remove_artifact(&ar("successor"));
        assert_eq!(
            res,
            Err(
                "Cannot safely remove successor because is referenced by number_5 and plus_2"
                    .into()
            )
        );
        model.validate_model().unwrap();
        assert!(model.list_artifacts().contains(&ar("zero")));
        assert!(model.list_artifacts().contains(&ar("number_4")));
    }

    #[test]
    fn can_rename_slot_of_unused_block() {
        let mut model = peano_model();
        model
            .rename_slot(&ar("successor"), &sn("x"), sn("y"))
            .expect("Should be able to rename it");
        assert!(!model
            .slots_of(&ar("successor"))
            .unwrap()
            .contains_key(&sn("x")));
        assert!(model
            .slots_of(&ar("successor"))
            .unwrap()
            .contains_key(&sn("y")));
    }

    #[test]
    fn cannot_rename_slot_into_an_already_existent_name() {
        // This includes renaming a slot to the same name for now
        let mut model = peano_model();
        let res = model.rename_slot(&ar("successor"), &sn("x"), sn("x"));

        assert_eq!(
            res,
            Err("Artifact successor already has a slot named 'x".into())
        );
        model.validate_model().unwrap();
        assert!(model
            .slots_of(&ar("successor"))
            .unwrap()
            .contains_key(&sn("x")));
    }

    #[test]
    fn cannot_rename_an_unexistent_slot() {
        let mut model = peano_model();
        let res = model.rename_slot(&ar("zero"), &sn("z"), sn("y"));

        assert_eq!(res, Err("Slot 'z does not exists in zero".into()));
        model.validate_model().unwrap();
        assert!(model.slots_of(&ar("zero")).unwrap().is_empty());
    }

    #[test]
    fn renaming_slot_name_changes_dependant_structures() {
        let mut model = two_and_two_is_four_model();
        model
            .rename_slot(&ar("successor"), &sn("x"), sn("y"))
            .expect("Should be able to rename it");
        model.validate_model().unwrap();
        assert!(!model
            .slots_of(&ar("successor"))
            .unwrap()
            .contains_key(&sn("x")));
        assert!(model
            .slots_of(&ar("successor"))
            .unwrap()
            .contains_key(&sn("y")));
        assert_eq!(model.slots_of(&ar("plus_2")).unwrap().len(), 1);
        assert!(model
            .slots_of(&ar("plus_2"))
            .unwrap()
            .contains_key(&sn("x")));
    }

    #[test]
    fn can_safely_remove_a_slot_if_its_artifact_is_not_used_by_any_structure() {
        let mut model = peano_model();
        model
            .safely_remove_block_slot(&ar("successor"), &sn("x"))
            .expect("Should be able to remove it");
        model.validate_model().unwrap();
        assert!(model.slots_of(&ar("successor")).unwrap().is_empty());
    }

    #[test]
    fn cannot_delete_an_unexisting_slot() {
        let mut model = peano_model();
        let res = model.safely_remove_block_slot(&ar("zero"), &sn("q"));
        assert_eq!(res, Err("Block zero does not have a 'q slot".into()));
        model.validate_model().unwrap();
        assert!(model
            .slots_of(&ar("successor"))
            .unwrap()
            .contains_key(&sn("x")));
    }

    #[test]
    fn cannot_delete_slot_from_an_unexistent_artifact() {
        let mut model = peano_model();
        let res = model.safely_remove_block_slot(&ar("wibblidy"), &sn("woob"));
        assert_eq!(res, Err("Artifact wibblidy does not exist".into()));
        model.validate_model().unwrap();
    }

    #[test]
    fn cannot_delete_slot_from_a_structure() {
        let mut model = and_one_more_is_five_model();
        let res = model.safely_remove_block_slot(&ar("plus_2"), &sn("x"));
        assert_eq!(res, Err("Cannot remove slot of a structure".into()));
        model.validate_model().unwrap();
    }

    #[test]
    fn cannot_delete_slot_if_some_structure_uses_it() {
        let mut model = and_one_more_is_five_model();
        let res = model.safely_remove_block_slot(&ar("successor"), &sn("x"));
        assert_eq!(
            res,
            Err(
                "Cannot remove slot 'x from successor because is used by number_5 and plus_2"
                    .into()
            )
        );
        model.validate_model().unwrap();
        assert!(model
            .slots_of(&ar("successor"))
            .unwrap()
            .contains_key(&sn("x")));
    }

    #[test]
    fn cannot_delete_slot_if_model_is_not_valid() {
        let mut model = two_and_two_is_four_model();
        model.remove_artifact(&ar("plus_2"));
        let res = model.safely_remove_block_slot(&ar("successor"), &sn("x"));
        assert_eq!(
            res,
            Err("Cannot safely remove slot in an invalid model".into())
        );
        assert!(model
            .slots_of(&ar("successor"))
            .unwrap()
            .contains_key(&sn("x")));
    }

    #[test]
    fn can_safely_remove_a_slot_if_is_not_used_in_any_strucure() {
        let mut model = two_and_two_is_four_model();
        model.set_artifact(
            ar("successor"),
            Artifact::Block(Block {
                main_slot_kind: sk("Natural"),
                slots: hashmap! {
                    sn("x") => sk("Natural"),
                    sn("step") => sk("Natural"),
                },
            }),
        );
        // TODO: Assert that model here is 'disconnected' but not that much invalid
        model
            .safely_remove_block_slot(&ar("successor"), &sn("step"))
            .expect("Should be able to remove it");
        model.validate_model().unwrap();
        assert!(model
            .slots_of(&ar("successor"))
            .unwrap()
            .contains_key(&sn("x")));
        assert_eq!(model.slots_of(&ar("successor")).unwrap().len(), 1);
    }
}

#[test]
#[ignore]
fn roadmap() {
    todo!("Separate concept of 'disconnected model' from the 'invalid model'");
    todo!("Define commands and events (results of those commands)");
    todo!("Check if a recursion can happen if the mutal recursion loop is longer (add breadcrumb to implementation and also improve errors)");

    todo!("Assert that structure expansion equals to an equivalent structure depending only on blocks");
}
