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
    // TODO: Change strings to domain types
    AddingABlock(String, String),
    ChangingBlockKind(ArtifactReference, String),
    AddingSlotToBlock(ArtifactReference, String, String),
    RenamingBlockSlot(ArtifactReference, SlotName, String),
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
    OpenChangeBlockKindPrompt(ArtifactReference),
    OpenAddSlotToBlockPrompt(ArtifactReference),
    ConfirmChangeBlockKind(ArtifactReference, SlotKind),
    ConfirmAddSlotToBlock(ArtifactReference, SlotName, SlotKind),
    OpenRenameBlockSlotPrompt(ArtifactReference, SlotName),
    ConfirmRenameBlockSlot(ArtifactReference, SlotName, SlotName),
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
            // TODO: EditorState might be a trait, but it needs to define a way of dealing with
            // nested editor states. This seems closely related to the idea of ui components, but
            // using egui in this case. For now, is better to wait and see which patterns appear in
            // the code.
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
            EditorState::ChangingBlockKind(aref, ref mut new_slot_kind) => {
                ui.label(format!("New main slot kind for {}:", aref));
                ui.text_edit_singleline(new_slot_kind);
                ui.separator();
                if ui.button("Cancel").clicked() {
                    context.should(EditorAction::GoToListing);
                }
                if ui.button("Change").clicked() {
                    context.should(EditorAction::ConfirmChangeBlockKind(
                        aref.clone(),
                        sk(&*new_slot_kind),
                    ));
                }
            }
            EditorState::AddingSlotToBlock(aref, ref mut slot_name, ref mut slot_kind) => {
                ui.label(format!("Name for the new slot in {}:", aref));
                ui.text_edit_singleline(slot_name);
                ui.label("Kind for the new slot:");
                ui.text_edit_singleline(slot_kind);
                ui.separator();
                if ui.button("Cancel").clicked() {
                    context.should(EditorAction::GoToListing);
                }
                if ui.button("Add").clicked() {
                    context.should(EditorAction::ConfirmAddSlotToBlock(
                        aref.clone(),
                        sn(&*slot_name),
                        sk(&*slot_kind)
                    ));
                }
            }
            EditorState::RenamingBlockSlot(ref aref, ref slot_name, ref mut slot_new_name) => {
                ui.label(format!("You are renaming {} slot in {}", slot_name, aref));
                ui.label("New name for the slot:");
                ui.text_edit_singleline(slot_new_name);
                ui.separator();
                if ui.button("Cancel").clicked() {
                    context.should(EditorAction::GoToListing);
                }
                if ui.button("Rename").clicked() {
                    context.should(EditorAction::ConfirmRenameBlockSlot(
                        aref.clone(),
                        slot_name.clone(),
                        sn(&*slot_new_name)
                    ));
                }
            }
        };

        ui.separator();
        if context.model_is_valid {
            ui.label("🌑 Model is valid");
        } else {
            ui.colored_label(Color32::RED, "🌕 Model is not valid");
        }
        if let Some(entry) = context.log.last() {
            match entry {
                EditorLog::Info(text) => ui.label(format!("💬 {}", text)),
                EditorLog::Error(text) => ui.colored_label(Color32::RED, format!("💢 {}", text)),
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
                    Artifact::Block(b) => self.detail_block(context, model, ui, aref, b),
                    Artifact::Structure(s) => self.detail_structure(context, model, ui, s),
                });
                ui.with_layout(Layout::top_down(Align::Max), |ui| {
                    self.actions_artifact(context, model, ui, aref);
                    match artifact {
                        Artifact::Block(b) => self.actions_block(context, model, ui, aref, b),
                        Artifact::Structure(s) => self.actions_stucture(context, model, ui, aref, s),
                    }
                })
            })
        });
    }

    fn actions_artifact(&mut self, context: &mut EditorContext, _model: &dyn Model, ui: &mut Ui, aref: &ArtifactReference) {
        if ui.small_button("delete").clicked() {
            context.should(EditorAction::SafelyDeleteArtifact(aref.clone()))
        }
    }

    fn actions_block(&mut self, context: &mut EditorContext, _model: &dyn Model, ui: &mut Ui, aref: &ArtifactReference, _block: &Block) {
        if ui.small_button("change kind").clicked() {
            context.should(EditorAction::OpenChangeBlockKindPrompt(aref.clone()))
        }
        if ui.small_button("add slot").clicked() {
            context.should(EditorAction::OpenAddSlotToBlockPrompt(aref.clone()))
        }
    }

    fn actions_stucture(&mut self, _context: &mut EditorContext, _model: &dyn Model, _ui: &mut Ui, _aref: &ArtifactReference, _structure: &Structure) {
    }

    fn detail_block(&mut self, context: &mut EditorContext, _model: &dyn Model, ui: &mut Ui, aref: &ArtifactReference, block: &Block) {
        ui.colored_label(Color32::LIGHT_BLUE, format!("{}", block.main_slot_kind));

        for (slot_name, slot_kind) in block.slots.iter() {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", slot_name));
                ui.colored_label(Color32::YELLOW, format!("{}", slot_kind));
                if ui.small_button("rename").clicked() {
                    context.should(EditorAction::OpenRenameBlockSlotPrompt(aref.clone(), slot_name.clone()));
                }
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
                            ui.colored_label(Color32::YELLOW, format!("{}", slot_kind));
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
            EditorState::ChangingBlockKind(_, _) => "Changing block kind",
            EditorState::AddingSlotToBlock(_, _, _) => "Adding slot to a block",
            EditorState::RenamingBlockSlot(_, _, _) => "Renaming block slot",
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
            EditorAction::OpenChangeBlockKindPrompt(aref) => {
                self.state =
                    EditorState::ChangingBlockKind(aref, "NewBlockKind".into());
            }
            EditorAction::ConfirmChangeBlockKind(aref, new_block_kind) => {
                // TODO: Getting the artifact to modify it and send it again modified is a bad
                // smell. This could be directly an action of the model.
                let block = match model.get_artifact(&aref) {
                    None => {
                        self.context.error(format!("Artifact {} does not exists", aref));
                        return;
                    },
                    Some(Artifact::Structure(_)) => {
                        self.context.error(format!("Artifact {} is not a block", aref));
                        return;
                    },
                    Some(Artifact::Block(block)) => {
                        block.clone()
                    }
                };
                model.set_artifact(aref.clone(), Artifact::Block(Block { main_slot_kind: new_block_kind.clone(), ..block.clone() }));

                if !model.is_all_valid() {
                    self.context.error(format!("Changing block kind would invalidate model"));
                    model.set_artifact(aref.clone(), Artifact::Block(block));
                    return;
                }

                self.context.info(format!("Changed {} kind to {}", aref, new_block_kind));
                self.state = EditorState::Listing;
            }
            EditorAction::OpenAddSlotToBlockPrompt(aref) => {
                self.state =
                    EditorState::AddingSlotToBlock(aref, "slot_name".into(), "SlotKind".into());
            }
            EditorAction::ConfirmAddSlotToBlock(aref, slot_name, slot_kind) => {
                // TODO: Getting the artifact to modify it and send it again modified is a bad
                // smell. This could be directly an action of the model. Plus, this is already
                // repeated code.
                let mut block = match model.get_artifact(&aref) {
                    None => {
                        self.context.error(format!("Artifact {} does not exists", aref));
                        return;
                    },
                    Some(Artifact::Structure(_)) => {
                        self.context.error(format!("Artifact {} is not a block", aref));
                        return;
                    },
                    Some(Artifact::Block(block)) => {
                        block.clone()
                    }
                };

                if block.slots.contains_key(&slot_name) {
                    self.context.error(format!("Slot {} already exists in {}", &slot_name, &aref));
                    return;
                }

                self.context.info(format!("Added slot {} of kind {} to {}", &slot_name, &slot_kind, &aref));
                block.slots.insert(slot_name, slot_kind);
                model.set_artifact(aref.clone(), Artifact::Block(block));
                // Yes, this operation most probably invalidates the model. 

                self.state = EditorState::Listing;
            }
            EditorAction::OpenRenameBlockSlotPrompt(aref, slot_name) => {
                self.state =
                    EditorState::RenamingBlockSlot(aref, slot_name, "new_slot_name".into());
            }
            EditorAction::ConfirmRenameBlockSlot(aref, slot_name, slot_new_name) => {
                todo!();
            }
        }
    }
}

/*
 * TODO:
 *
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
