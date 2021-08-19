use super::mursten::*;
use bevy_egui::egui::*;

#[derive(Debug, Default)]
pub struct ModelEditor {
    focused: bool,
    model_is_valid: bool,
    status: EditorStatus,
}

#[derive(Debug)]
enum EditorStatus {
    Listing,
    AddingABlock(String, String),
}

impl Default for EditorStatus {
    fn default() -> Self {
        Self::Listing
    }
}

pub enum EditorAction {
    Nothing,
    GoToListing,
    OpenAddBlockPrompt,
    ConfirmAddBlock(String, String),
}

impl ModelEditor {
    pub fn title(&mut self) -> &str {
        match self.status {
            EditorStatus::Listing => "Showing existing models",
            EditorStatus::AddingABlock(_, _) => "Adding a new block",
        }
    }
    pub fn show(&mut self, model: &dyn Model, ui: &mut Ui) -> EditorAction {
        self.model_is_valid = model.validate_model().is_ok();
        match self.status {
            EditorStatus::Listing => {
                for aref in model.list_artifacts().iter() {
                    self.list_item(model, ui, aref);
                }
                if self.model_is_valid {
                    ui.label("Model is valid");
                } else {
                    ui.colored_label(Color32::RED, "Model is not valid");
                }
                if ui.button("Add block").clicked() {
                    return EditorAction::OpenAddBlockPrompt;
                }
                EditorAction::Nothing
            }
            EditorStatus::AddingABlock(ref mut aref, ref mut slot_kind) => {
                ui.label("Artifact reference for the new block:");
                ui.text_edit_singleline(aref);
                ui.label("Main slot kind:");
                ui.text_edit_singleline(slot_kind);
                ui.separator();
                if ui.button("Cancel").clicked() {
                    return EditorAction::GoToListing;
                }
                if ui.button("Add").clicked() {
                    return EditorAction::ConfirmAddBlock(aref.clone(), slot_kind.clone());
                }
                EditorAction::Nothing
            }
        }
    }

    pub fn apply(&mut self, model: &dyn Model, changes: &mut Vec<ModelChange>, action: EditorAction)
    {
        match action {
            EditorAction::Nothing => (),
            EditorAction::GoToListing => {
                self.status = EditorStatus::Listing;
            }
            EditorAction::OpenAddBlockPrompt => {
                self.status = EditorStatus::AddingABlock("my_new_block".into(), "NewBlockKind".into());
            },
            EditorAction::ConfirmAddBlock(aref, slot_kind) => {
                changes.push(ModelChange::AddBlock(ar(aref), sk(slot_kind)));
                self.status = EditorStatus::Listing;
            }
        }
    }

    fn list_item(&mut self, model: &dyn Model, ui: &mut Ui, aref: &ArtifactReference) {
        let artifact = model.get_artifact(aref).unwrap();
        let prefix = match artifact {
            Artifact::Block(_) => "[B]",
            Artifact::Structure(_) => "[S]",
        };

        ui.collapsing(format!("{} {}", prefix, aref), |ui| match artifact {
            Artifact::Block(b) => self.detail_block(model, ui, b),
            Artifact::Structure(s) => self.detail_structure(model, ui, s),
        });
    }

    fn detail_block(&mut self, model: &dyn Model, ui: &mut Ui, block: &Block) {
        ui.colored_label(Color32::LIGHT_BLUE, format!("{}", block.main_slot_kind));

        for (slot_name, slot_kind) in block.slots.iter() {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", slot_name));
                ui.colored_label(Color32::RED, format!("{}", slot_kind));
            });
        }
    }

    fn detail_structure(&mut self, model: &dyn Model, ui: &mut Ui, structure: &Structure) {
        if self.model_is_valid {
            let main_slot_kind = model.main_slot_kind_of(&structure.a_ref).unwrap();
            ui.colored_label(Color32::LIGHT_BLUE, format!("{}", main_slot_kind));
        }
        ui.label(format!("{}", structure.a_ref));
        for (slot_name, c) in structure.c.iter() {
            match c {
                Connection::Slot(s) => {
                    if self.model_is_valid {
                        let slots = model.slots_of(&structure.a_ref).unwrap();
                        let (_, slot_kind) =
                            slots.into_iter().find(|(sn, _)| sn == slot_name).unwrap();
                        ui.horizontal(|ui| {
                            ui.label(format!("{} <- {}:", slot_name, s));
                            ui.colored_label(Color32::RED, format!("{}", slot_kind));
                        });
                    } else {
                        ui.label(format!("{} <- {}", slot_name, s));
                    }
                }
                Connection::Structure(s) => {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} <-", slot_name));
                        ui.separator();
                        ui.vertical(|ui| {
                            self.detail_structure(model, ui, s);
                        });
                    });
                }
            };
        }
    }
}

/*
 * TODO:
 * - create a block
 * - (safely) delete an artifact
 *
 * - change the type of a block
 * - add a slot to a block
 * - rename a slot of a block
 * - delete a slot from a block
 *
 * - see disonnected slots
 *
 * - create a structure starting from another artifact
 *
 * - rename a slot of a structure
 * - connect an artifact to a slot
 * - disconnect an artifact from a slot
 *
 * - be able to (safely) edit a structure
 *     - change artifact reference
 *     - connect artifact in slot
 */
