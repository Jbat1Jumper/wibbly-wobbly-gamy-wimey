use super::mursten::*;
use bevy_egui::egui::*;

#[derive(Debug, Default)]
pub struct ModelEditor {}

impl ModelEditor {
    pub fn show(&mut self, model: &dyn Model, ui: &mut Ui) {
        for aref in model.list_artifacts().iter() {
            self.list_item(model, ui, aref);
        }

        ui.label("Showing model");
    }
    fn list_item(&mut self, model: &dyn Model, ui: &mut Ui, aref: &ArtifactReference) {
        match model.get_artifact(aref).unwrap() {
            Artifact::Block(_) => ui.label(format!("[B] {}", aref)),
            Artifact::Structure(_) => ui.label(format!("[S] {}", aref)),
        };
    }
}

/*
 * TODO:
 * - be able to explore a model
 *     - list all available artifacts
 *     - see artifact detail (block or structure)
 *     - follow structure links
 *     - go back
 * - be able to create an artifact
 * - be able to (safely) delete an artifact
 * - be able to (safely) edit a structure
 *     - change artifact reference
 *     - connect artifact in slot
 */
