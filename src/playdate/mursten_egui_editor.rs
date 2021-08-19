use super::mursten::*;
use bevy_egui::egui::*;

#[derive(Debug, Default)]
pub struct ModelEditor {
    focused: bool,
    model_is_valid: bool,
    status_stack: Vec<EditorStatus>,
}

#[derive(Debug)]
enum EditorStatus {
    Listing,
}

impl ModelEditor {
    pub fn show(&mut self, model: &dyn Model, ui: &mut Ui) {
        self.model_is_valid = model.validate_model().is_ok();

        for aref in model.list_artifacts().iter() {
            self.list_item(model, ui, aref);
        }

        ui.label("Showing model");
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
                        let (_, slot_kind) = slots.into_iter().find(|(sn, _)| sn == slot_name).unwrap();
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
 * - be able to explore a model
 *     - follow structure links
 *     - go back
 * - be able to create an artifact
 * - be able to (safely) delete an artifact
 * - be able to (safely) edit a structure
 *     - change artifact reference
 *     - connect artifact in slot
 */
