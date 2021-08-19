use std::collections::HashMap;

use super::mursten::*;
use bevy_egui::egui::*;

#[derive(Debug, Default)]
pub struct ModelEditor {
    context: EditorContext,
    state: EditorState,
}

#[derive(Debug, Default)]
pub struct EditorContext {
    focused: bool,
    log: Vec<EditorLog>,
    model_is_valid: bool,
    pending_actions: Vec<EditorAction>,
}

#[derive(Debug)]
enum EditorState {
    Listing,
    AddingABlock(String, String),
}

impl Default for EditorState {
    fn default() -> Self {
        Self::Listing
    }
}

#[derive(Debug)]
pub enum EditorAction {
    GoToListing,
    OpenAddBlockPrompt,
    ConfirmAddBlock(String, String),
    SafelyDeleteArtifact(ArtifactReference),
}

#[derive(Debug)]
enum EditorLog {
    Info(String),
    Error(String),
}

impl EditorContext {
    pub fn error(&mut self, text: String) {
        self.log.push(EditorLog::Error(text));
    }

    pub fn info(&mut self, text: String) {
        self.log.push(EditorLog::Info(text));
    }

    fn should(&mut self, action: EditorAction) {
        self.pending_actions.push(action);
    }
}

impl EditorState {
    pub fn show(&mut self, context: &mut EditorContext, model: &dyn Model, ui: &mut Ui) {
        context.model_is_valid = model.validate_model().is_ok();
        match self {
            EditorState::Listing => {
                if ui.button("Add block").clicked() {
                    context.should(EditorAction::OpenAddBlockPrompt);
                }
                ui.separator();
                for aref in model.list_artifacts().iter() {
                    self.list_item(context, model, ui, aref);
                }
            }
            EditorState::AddingABlock(ref mut aref, ref mut slot_kind) => {
                ui.label("Artifact reference for the new block:");
                ui.text_edit_singleline(aref);
                ui.label("Main slot kind:");
                ui.text_edit_singleline(slot_kind);
                ui.separator();
                if ui.button("Cancel").clicked() {
                    context.should(EditorAction::GoToListing);
                }
                if ui.button("Add").clicked() {
                    context.should(EditorAction::ConfirmAddBlock(
                        aref.clone(),
                        slot_kind.clone(),
                    ));
                }
            }
        };

        ui.separator();
        if context.model_is_valid {
            ui.label("Model is valid");
        } else {
            ui.colored_label(Color32::RED, "Model is not valid");
        }
        if let Some(entry) = context.log.last() {
            match entry {
                EditorLog::Info(text) => ui.label(text),
                EditorLog::Error(text) => ui.colored_label(Color32::RED, text),
            };
        }
    }

    fn list_item(&mut self, context: &mut EditorContext, model: &dyn Model, ui: &mut Ui, aref: &ArtifactReference) {
        let artifact = model.get_artifact(aref).unwrap();
        let prefix = match artifact {
            Artifact::Block(_) => "[B]",
            Artifact::Structure(_) => "[S]",
        };

        ui.collapsing(format!("{} {}", prefix, aref), |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| match artifact {
                    Artifact::Block(b) => self.detail_block(context, model, ui, b),
                    Artifact::Structure(s) => self.detail_structure(context, model, ui, s),
                });
                ui.with_layout(Layout::top_down(Align::Max), |ui| {
                    if ui.small_button("T").clicked() {
                        context.should(EditorAction::SafelyDeleteArtifact(aref.clone()))
                    }
                })
            })
        });
    }

    fn detail_block(&mut self, context: &mut EditorContext, model: &dyn Model, ui: &mut Ui, block: &Block) {
        ui.colored_label(Color32::LIGHT_BLUE, format!("{}", block.main_slot_kind));

        for (slot_name, slot_kind) in block.slots.iter() {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", slot_name));
                ui.colored_label(Color32::RED, format!("{}", slot_kind));
            });
        }
    }

    fn detail_structure(&mut self, context: &mut EditorContext, model: &dyn Model, ui: &mut Ui, structure: &Structure) {
        if context.model_is_valid {
            let main_slot_kind = model.main_slot_kind_of(&structure.a_ref).unwrap();
            ui.colored_label(Color32::LIGHT_BLUE, format!("{}", main_slot_kind));
        }
        ui.label(format!("{}", structure.a_ref));
        for (slot_name, c) in structure.c.iter() {
            match c {
                Connection::Slot(s) => {
                    if context.model_is_valid {
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
                            self.detail_structure(context, model, ui, s);
                        });
                    });
                }
            };
        }
    }
}

impl ModelEditor {
    pub fn title(&mut self) -> &str {
        match self.state {
            EditorState::Listing => "Showing existing models",
            EditorState::AddingABlock(_, _) => "Adding a new block",
        }
    }

    pub fn show(&mut self, model: &dyn Model, ui: &mut Ui) {
        let Self {
            ref mut state,
            ref mut context,
        } = self;
        state.show(context, model, ui);
    }

    pub fn fullfill_actions(&mut self, model: &mut dyn Model) {
        for action in self.context.pending_actions.drain(..).collect::<Vec<_>>() {
            self.fullfill_action(model, action);
        }
    }
    pub fn fullfill_action(
        &mut self,
        model: &mut dyn Model,
        action: EditorAction,
    ) {
        match action {
            EditorAction::GoToListing => {
                self.state = EditorState::Listing;
            }
            EditorAction::OpenAddBlockPrompt => {
                self.state =
                    EditorState::AddingABlock("my_new_block".into(), "NewBlockKind".into());
            }
            EditorAction::ConfirmAddBlock(ref aref, ref slot_kind) => {
                if model.exists_artifact(&ar(aref)) {
                    self.context.error(format!("Block {} already exists", aref));
                    return;
                }
                model.set_artifact(
                    ar(aref),
                    Artifact::Block(Block {
                        main_slot_kind: sk(slot_kind),
                        slots: HashMap::default(),
                    }),
                );
                self.context.info(format!("Added block {} with kind {}", aref, slot_kind));
                self.state = EditorState::Listing;
            }
            EditorAction::SafelyDeleteArtifact(aref) => {
                if !model.exists_artifact(&aref) {
                    self.context.error(format!("Block {} does not exists", aref));
                    return;
                }
                let deps = model.dependents(&aref);
                if deps.is_empty() {
                    model.remove_artifact(&aref);
                    self.context.info(format!("Removed {}", aref))
                } else {
                    self.context.error(format!(
                        "Cannot delete {}, because {} depends on it",
                        aref,
                        deps.into_iter()
                            .map(|dependent_aref| dependent_aref.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                }
            }
        }
    }
}

/*
 * TODO:
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
