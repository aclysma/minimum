use std::collections::{HashSet, HashMap, VecDeque};
use legion::prelude::*;
use legion::storage::ComponentTypeId;
use minimum_kernel::resources::{AssetResource, ComponentRegistryResource};
use minimum_kernel::pipeline::PrefabAsset;
use minimum_game::resources::{TimeResource, UniverseResource};
use crate::resources::EditorSelectionResource;
use minimum_game::resources::SimulationTimePauseReason;
use atelier_assets::core::AssetUuid;
use legion_prefab::{CookedPrefab, ComponentRegistration, Prefab};
use std::sync::Arc;
use minimum_game::resources::TimeState;
use atelier_assets::loader::handle::{TypedAssetStorage, AssetHandle};
use legion_transaction::{ComponentDiff, apply_diff_to_prefab, WorldDiff};
use prefab_format::{ComponentTypeUuid, EntityUuid};
use std::collections::vec_deque;
use legion_transaction::CopyCloneImpl;
use legion_transaction::{TransactionBuilder, TransactionDiffs, TransactionEntityInfo, Transaction};
use imgui::ImString;

use atelier_assets::loader as atelier_loader;
use minimum_kernel::ComponentRegistry;

#[derive(Clone, Copy)]
pub enum PostCommitSelection {
    /// At the end of the transaction, do not change what entities are selected
    KeepCurrentSelection,

    /// At the end of the transaction, select all the entities that were included in the transaction
    SelectAllInTransaction,
}

/// Operations that can be performed in the editor. These get queued up to be executed later at a
/// single place in the frame in FIFO order
enum EditorOp {
    /// Clear the world and load the given prefab into it
    OpenPrefab(AssetUuid),

    /// Save the current pre-play state to the currently open prefab file
    SavePrefab,

    /// Unpauses the simulation, allowing in-editor testing
    Play,

    /// Pause the simulation
    Pause,

    /// Play/pause the simulation (see Play and Pause ops)
    TogglePause,

    /// Pause the simulation and revert everything back to pre-play state
    Reset,

    /// Undo the previous change
    Undo,

    /// Redo a change that was previously undone
    Redo,

    /// Sets the current editor tool (translate, scale, etc.)
    SetActiveEditorTool(EditorTool),
}

/// Tracks which windows are open
pub struct WindowOptions {
    pub show_imgui_metrics: bool,
    pub show_imgui_style_editor: bool,
    pub show_imgui_demo: bool,
    pub show_entity_list: bool,
    pub show_inspector: bool,
}

impl WindowOptions {
    pub fn new() -> Self {
        WindowOptions {
            show_imgui_metrics: false,
            show_imgui_style_editor: false,
            show_imgui_demo: false,
            show_entity_list: false,
            show_inspector: false,
        }
    }

    pub fn new_runtime() -> Self {
        let mut options = Self::new();
        options.show_entity_list = true;
        options.show_inspector = true;
        options
    }

    pub fn new_editing() -> Self {
        let mut options = Self::new();
        options.show_entity_list = true;
        options.show_inspector = true;
        options
    }
}

// If adding to this, don't forget to hook up keyboard shortcuts and buttons
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum EditorTool {
    Translate,
    Scale,
    Rotate,
}

#[derive(PartialEq, Debug, Copy, Clone, Eq, Hash)]
pub enum EditorMode {
    Inactive,
    Active,
}

/// The data we track that's associated with a prefab being opened
pub struct OpenedPrefabState {
    /// UUID of the asset we are editing
    uuid: AssetUuid,

    /// The version that was loaded. This is compared against the version located in AssetStorage.
    /// If the versions don't match, we reload the data
    version: u32,

    /// Handle to the opened prefab, used to reload data if a new version arrives (possibly by file
    /// on disk changing)
    prefab_handle: atelier_loader::handle::Handle<PrefabAsset>,

    /// The opened prefab in uncooked form. Any diffs that are applied to the world also get applied
    /// to the prefab and cooked prefab so that when we save data, we can just persist this field.
    /// Long-term, the responsibility for this would be on the asset daemon
    uncooked_prefab: Arc<Prefab>,

    /// The opened prefab in cooked form. This is used for reloads and applying edits against
    cooked_prefab: Arc<CookedPrefab>,

    /// Assists in finding the world entity that corresponds with a prefab entity
    prefab_to_world_mappings: HashMap<Entity, Entity>,

    /// Assists in finding the prefab entity that corresponds with a world entity
    world_to_prefab_mappings: HashMap<Entity, Entity>,
}

impl OpenedPrefabState {
    pub fn cooked_prefab(&self) -> &Arc<CookedPrefab> {
        &self.cooked_prefab
    }

    pub fn prefab_to_world_mappings(&self) -> &HashMap<Entity, Entity> {
        &self.prefab_to_world_mappings
    }

    pub fn world_to_prefab_mappings(&self) -> &HashMap<Entity, Entity> {
        &self.world_to_prefab_mappings
    }

    pub fn uuid(&self) -> &AssetUuid {
        &self.uuid
    }
}

/// Diffs that are pending being applied
struct TransactionDiffsPendingApply {
    /// The diffs required to apply/revert the transaction
    diffs: TransactionDiffs,

    /// If true, an undo step will be recorded
    commit_changes: bool,

    /// How to handle selection after the transaction commits
    post_commit_selection: PostCommitSelection,
}

/// Contains the data required to identify the current transaction by ID and commit or cancel the
/// transaction
struct CurrentTransactionInfo {
    /// The ID of the transaction that is currently in progress
    id: EditorTransactionId,

    /// The diffs required to commit or cancel the transaction (apply vs. revert)
    diffs: TransactionDiffs,
}

pub struct EditorStateResource {
    // Indicates the overall state of the editor (i.e. editing vs. playing)
    editor_mode: EditorMode,

    // Runtime state for editing UI
    window_options_running: WindowOptions,
    window_options_editing: WindowOptions,
    active_editor_tool: EditorTool,
    pub add_component_search_text: ImString,

    // If a prefab is opened, this holds the state associated with editing it
    opened_prefab: Option<Arc<OpenedPrefabState>>,

    // We queue important operations to happen as many of them require taking fairly invasive
    // mut references to the world and resources. Each frame we drain this and execute each
    // operation
    pending_editor_ops: Vec<EditorOp>,

    // Editor transaction will enqueue diffs here to be applied to the world. These are drained
    // each frame, applied to the world state, and possibly inserted into the undo queue
    diffs_pending_apply: Vec<TransactionDiffsPendingApply>,

    // Undo/redo steps. Each slot in the chain contains diffs to go forward/backward in the
    // chain.
    undo_chain: VecDeque<Arc<TransactionDiffs>>,
    undo_chain_position: usize,

    // The current transaction for any sort of gizmo interaction (draging to change
    // position, rotation, scaling)
    gizmo_transaction: Option<EditorTransaction>,

    // If a transaction is in progress, the data required to identify it and commit it is
    // stored here. The ID is used to determine if a transaction provided by downstream code
    // is the same as the one that's currently in progress. If it isn't the same, we commit
    // the old transaction and accept the new one. This inserts a new entry in the undo
    // chain
    current_transaction_info: Option<CurrentTransactionInfo>,
}

impl EditorStateResource {
    pub fn new() -> Self {
        EditorStateResource {
            editor_mode: EditorMode::Inactive,
            window_options_running: WindowOptions::new_runtime(),
            window_options_editing: WindowOptions::new_editing(),
            active_editor_tool: EditorTool::Translate,
            add_component_search_text: ImString::with_capacity(255),
            opened_prefab: None,
            pending_editor_ops: Default::default(),

            diffs_pending_apply: Default::default(),

            undo_chain: Default::default(),
            undo_chain_position: 0,

            gizmo_transaction: None,

            current_transaction_info: None,
        }
    }

    pub fn opened_prefab(&self) -> Option<Arc<OpenedPrefabState>> {
        self.opened_prefab.clone()
    }

    pub fn is_editor_active(&self) -> bool {
        self.editor_mode != EditorMode::Inactive
    }

    pub fn editor_mode(&self) -> EditorMode {
        self.editor_mode
    }

    pub fn window_options(&self) -> &WindowOptions {
        if self.is_editor_active() {
            &self.window_options_editing
        } else {
            &self.window_options_running
        }
    }

    pub fn window_options_mut(&mut self) -> &mut WindowOptions {
        if self.is_editor_active() {
            &mut self.window_options_editing
        } else {
            &mut self.window_options_running
        }
    }

    pub fn gizmo_transaction(&self) -> &Option<EditorTransaction> {
        &self.gizmo_transaction
    }

    pub fn gizmo_transaction_mut(&mut self) -> &mut Option<EditorTransaction> {
        &mut self.gizmo_transaction
    }

    fn play(
        &mut self,
        time_state: &mut TimeResource,
    ) {
        self.editor_mode = EditorMode::Inactive;
        time_state.set_simulation_time_paused(false, SimulationTimePauseReason::Editor);
    }

    fn pause(
        &mut self,
        time_state: &mut TimeResource,
    ) {
        self.editor_mode = EditorMode::Active;
        time_state.set_simulation_time_paused(true, SimulationTimePauseReason::Editor);
    }

    pub fn toggle_pause(
        &mut self,
        time_state: &mut TimeResource,
    ) {
        match self.editor_mode {
            EditorMode::Active => self.play(time_state),
            EditorMode::Inactive => self.pause(time_state),
        };
    }

    pub fn open_prefab(
        world: &mut World,
        resources: &Resources,
        prefab_uuid: AssetUuid,
    ) {
        {
            let mut asset_resource = resources.get_mut::<AssetResource>().unwrap();

            use atelier_assets::loader::Loader;
            use atelier_assets::loader::handle::AssetHandle;

            let load_handle = asset_resource.loader().add_ref(prefab_uuid);
            let handle = atelier_loader::handle::Handle::<PrefabAsset>::new(
                asset_resource.tx().clone(),
                load_handle,
            );

            let version = loop {
                asset_resource.update();
                if let atelier_loader::LoadStatus::Loaded = handle
                    .load_status::<atelier_loader::rpc_loader::RpcLoader>(
                    asset_resource.loader(),
                ) {
                    break handle
                        .asset_version::<PrefabAsset, _>(asset_resource.storage())
                        .unwrap();
                }
            };

            let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();

            // Load the uncooked prefab from disk and cook it. (Eventually this will be handled
            // during atelier's build step
            let mut universe = resources.get_mut::<UniverseResource>().unwrap();
            let component_registry = resources.get::<ComponentRegistryResource>().unwrap();
            let cooked_prefab = Arc::new(minimum_kernel::prefab_cooking::cook_prefab(
                &*universe,
                &mut *asset_resource,
                component_registry.components(),
                component_registry.components_by_uuid(),
                prefab_uuid,
            ));

            let component_registry = resources.get::<ComponentRegistryResource>().unwrap();

            // Duplicate the prefab data so we can apply diffs to it. This is temporary and will eventually be
            // done within the daemon. (This is kind of like a clone() on the uncooked prefab asset)
            let noop_diff = WorldDiff::new(vec![], vec![]);
            let uncooked_prefab = Arc::new(legion_transaction::apply_diff_to_prefab(
                &handle.asset(asset_resource.storage()).unwrap().prefab,
                &universe.universe,
                &noop_diff,
                component_registry.components_by_uuid(),
                &component_registry.copy_clone_impl(),
            ));

            // Store the cooked prefab and relevant metadata in an Arc on the EditorStateResource.
            // Eventually the cooked prefab data would be held by AssetStorage and we'd just hold
            // a handle to it.
            let opened_prefab = OpenedPrefabState {
                uuid: prefab_uuid,
                version,
                prefab_handle: handle,
                uncooked_prefab,
                cooked_prefab,
                prefab_to_world_mappings: Default::default(),
                world_to_prefab_mappings: Default::default(),
            };

            editor_state.opened_prefab = Some(Arc::new(opened_prefab));
        }

        Self::reset(world, resources);
    }

    fn reset(
        world: &mut World,
        resources: &Resources,
    ) {
        log::info!("RESET THE WORLD");
        // this is scoped to avoid holding TimeResource while spawning
        {
            let mut time_resource = resources.get_mut::<TimeResource>().unwrap();
            time_resource.set_simulation_time_paused(true, SimulationTimePauseReason::Editor);
            time_resource.reset_simulation_time();
        }

        // Clone the Arc containing all relevant data about the prefab we're currently editing
        // this is scoped to avoid holding EditorStateResource while spawning
        let opened_prefab = {
            let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
            editor_state.editor_mode = EditorMode::Active;
            editor_state.opened_prefab.clone()
        };

        // If a prefab is opened, reset all the data
        if let Some(opened_prefab) = opened_prefab {
            let mut prefab_to_world_mappings = HashMap::default();
            let component_registry = resources.get::<ComponentRegistryResource>().unwrap();
            world.clone_from(
                &opened_prefab.cooked_prefab.world,
                &component_registry.spawn_clone_impl(resources),
                &mut legion::world::HashMapCloneImplResult(&mut prefab_to_world_mappings),
                &legion::world::HashMapEntityReplacePolicy(&opened_prefab.prefab_to_world_mappings),
            );

            let mut world_to_prefab_mappings =
                HashMap::with_capacity(prefab_to_world_mappings.len());
            for (k, v) in &prefab_to_world_mappings {
                world_to_prefab_mappings.insert(*v, *k);
            }

            for (cooked_prefab_entity_uuid, cooked_prefab_entity) in
                &opened_prefab.cooked_prefab.entities
            {
                let world_entity = prefab_to_world_mappings.get(cooked_prefab_entity);
                log::info!(
                    "Prefab entity {} {:?} spawned as world entity {:?}",
                    uuid::Uuid::from_bytes(*cooked_prefab_entity_uuid).to_string(),
                    cooked_prefab_entity,
                    world_entity
                );
            }

            let new_opened_prefab = OpenedPrefabState {
                uuid: opened_prefab.uuid,
                cooked_prefab: opened_prefab.cooked_prefab.clone(),
                prefab_handle: opened_prefab.prefab_handle.clone(),
                uncooked_prefab: opened_prefab.uncooked_prefab.clone(),
                version: opened_prefab.version,
                prefab_to_world_mappings,
                world_to_prefab_mappings,
            };

            let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
            editor_state.opened_prefab = Some(Arc::new(new_opened_prefab));
        }
    }

    pub fn active_editor_tool(&self) -> EditorTool {
        self.active_editor_tool
    }

    pub fn enqueue_save_prefab(&mut self) {
        self.pending_editor_ops.push(EditorOp::SavePrefab);
    }

    pub fn enqueue_play(&mut self) {
        self.pending_editor_ops.push(EditorOp::Play);
    }

    pub fn enqueue_pause(&mut self) {
        self.pending_editor_ops.push(EditorOp::Pause);
    }

    pub fn enqueue_reset(&mut self) {
        self.pending_editor_ops.push(EditorOp::Reset);
    }

    pub fn enqueue_open_prefab(
        &mut self,
        prefab_uuid: AssetUuid,
    ) {
        self.pending_editor_ops
            .push(EditorOp::OpenPrefab(prefab_uuid));
    }

    pub fn enqueue_toggle_pause(&mut self) {
        self.pending_editor_ops.push(EditorOp::TogglePause);
    }

    pub fn enqueue_undo(&mut self) {
        self.pending_editor_ops.push(EditorOp::Undo);
    }

    pub fn enqueue_redo(&mut self) {
        self.pending_editor_ops.push(EditorOp::Redo);
    }

    pub fn enqueue_set_active_editor_tool(
        &mut self,
        editor_tool: EditorTool,
    ) {
        self.pending_editor_ops
            .push(EditorOp::SetActiveEditorTool(editor_tool));
    }

    pub fn set_active_editor_tool(
        &mut self,
        editor_tool: EditorTool,
    ) {
        self.active_editor_tool = editor_tool;
        log::info!("Editor tool changed to {:?}", editor_tool);
    }

    pub fn process_editor_ops(
        world: &mut World,
        resources: &Resources,
    ) {
        let editor_ops: Vec<_> = resources
            .get_mut::<EditorStateResource>()
            .unwrap()
            .pending_editor_ops
            .drain(..)
            .collect();
        for editor_op in editor_ops {
            match editor_op {
                EditorOp::OpenPrefab(asset_uuid) => {
                    let new_world = {
                        let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
                        editor_state.clear_undo_history();

                        let universe = resources.get::<UniverseResource>().unwrap();
                        let world = universe.universe.create_world();
                        world
                    };
                    *world = new_world;
                    Self::open_prefab(world, resources, asset_uuid)
                }
                EditorOp::SavePrefab => {
                    let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
                    let mut component_registry = resources.get_mut::<ComponentRegistry>().unwrap();
                    editor_state.save(&*component_registry);
                }
                EditorOp::Play => {
                    let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
                    let mut time_state = resources.get_mut::<TimeResource>().unwrap();
                    editor_state.play(&mut *time_state)
                }
                EditorOp::Pause => {
                    let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
                    let mut time_state = resources.get_mut::<TimeResource>().unwrap();
                    editor_state.pause(&mut *time_state)
                }
                EditorOp::Reset => Self::reset(world, resources),
                EditorOp::TogglePause => {
                    let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
                    let mut time_state = resources.get_mut::<TimeResource>().unwrap();
                    editor_state.toggle_pause(&mut *time_state)
                }
                EditorOp::SetActiveEditorTool(editor_tool) => {
                    let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
                    editor_state.set_active_editor_tool(editor_tool)
                }
                EditorOp::Undo => {
                    Self::undo(world, resources);
                }
                EditorOp::Redo => {
                    Self::redo(world, resources);
                }
            }
        }
    }

    fn get_selected_uuids(
        &mut self,
        selection_resource: &EditorSelectionResource,
        world: &World,
    ) -> HashSet<EntityUuid> {
        // Get the UUIDs of all selected entities
        let mut selected_uuids = HashSet::new();

        if let Some(opened_prefab) = self.opened_prefab() {
            // Reverse the keys/values of the opened prefab map so we can efficiently look up the UUID of entities in the prefab
            use std::iter::FromIterator;
            let prefab_entity_to_uuid: HashMap<Entity, prefab_format::EntityUuid> =
                HashMap::from_iter(
                    opened_prefab
                        .cooked_prefab()
                        .entities
                        .iter()
                        .map(|(k, v)| (*v, *k)),
                );

            // Iterate all selected prefab entities
            for selected_entity in selection_resource.selected_entities() {
                if let Some(prefab_entity) =
                    opened_prefab.world_to_prefab_mappings.get(selected_entity)
                {
                    let entity_uuid = prefab_entity_to_uuid.get(prefab_entity);
                    // Insert the UUID into selected_uuids
                    if let Some(uuid) = entity_uuid {
                        log::info!(
                            "Selected entity {:?} corresponds to prefab entity {:?} uuid {:?}",
                            selected_entity,
                            prefab_entity,
                            uuid::Uuid::from_bytes(*uuid).to_string()
                        );
                        selected_uuids.insert(*uuid);
                    } else {
                        //TODO: For now this is a panic because it really shouldn't happen and we want to make sure it's visible if it does, but
                        // losing selection info shouldn't be fatal
                        panic!("Could not find prefab entity {:?} which should have corresponded with selected entity {:?}", prefab_entity, selected_entity);
                    }
                }
            }
        }
        selected_uuids
    }

    fn restore_selected_uuids(
        &mut self,
        selection_resource: &mut EditorSelectionResource,
        world: &World,
        selected_uuids: &HashSet<EntityUuid>,
    ) {
        let mut selected_entities: HashSet<Entity> = HashSet::default();
        for selected_uuid in selected_uuids {
            if let Some(opened_prefab) = self.opened_prefab.as_ref() {
                if let Some(prefab_entity) =
                    &opened_prefab.cooked_prefab.entities.get(selected_uuid)
                {
                    let world_entity = opened_prefab.prefab_to_world_mappings[prefab_entity];
                    selected_entities.insert(world_entity);
                }
            }
        }

        selection_resource.enqueue_set_selection(selected_entities.into_iter().collect());
    }

    pub fn hot_reload_if_asset_changed(
        world: &mut World,
        resources: &Resources,
    ) {
        // Detect if we need to reload. Do this comparing the prefab asset's version with the cooked prefab's version
        let mut prefab_to_reload = None;
        {
            let editor_state = resources.get::<EditorStateResource>().unwrap();
            if let Some(opened_prefab) = &editor_state.opened_prefab {
                let mut asset_resource = resources.get_mut::<AssetResource>().unwrap();
                let version = opened_prefab
                    .prefab_handle
                    .asset_version::<PrefabAsset, _>(asset_resource.storage())
                    .unwrap();
                if opened_prefab.version != version {
                    prefab_to_reload = Some(opened_prefab.clone());
                }
            }
        }

        // If prefab_to_reload is not none, do the reload
        if let Some(opened_prefab) = prefab_to_reload {
            log::info!("Source file change detected, reloading");

            // Save the selected entity UUIDs
            let selected_uuids = {
                let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
                let selection_resource = resources.get::<EditorSelectionResource>().unwrap();
                editor_state.get_selected_uuids(&*selection_resource, world)
            };

            // Delete the old stuff from the world
            for x in opened_prefab.prefab_to_world_mappings.values() {
                world.delete(*x);
            }

            // re-cook and load the prefab
            Self::open_prefab(world, resources, opened_prefab.uuid);

            // Restore selection
            let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
            let mut selection_resource = resources.get_mut::<EditorSelectionResource>().unwrap();
            editor_state.restore_selected_uuids(&mut *selection_resource, world, &selected_uuids);
        }
    }

    pub fn enqueue_diffs(
        &mut self,
        diffs: TransactionDiffs,
        commit_changes: bool,
        post_commit_selection: PostCommitSelection,
    ) {
        if diffs.apply_diff().has_changes() {
            self.diffs_pending_apply.push(TransactionDiffsPendingApply {
                diffs,
                commit_changes,
                post_commit_selection,
            });
        }
    }

    pub fn process_diffs(
        world: &mut World,
        resources: &mut Resources,
    ) {
        // Flush selection ops grab all the diffs pending apply
        let mut diffs_pending_apply = vec![];
        {
            // These are scoped so they won't be borrowed when calling apply_diffs
            let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
            let mut editor_selection = resources.get_mut::<EditorSelectionResource>().unwrap();

            // flush selection ops, world entities can change after this call leading to entities not being
            // found and selections lost
            let universe_resource = resources.get::<UniverseResource>().unwrap();
            editor_selection.process_selection_ops(&mut *editor_state, &*universe_resource, world);

            // Take all the diffs that are queued to be applied this frame
            if !editor_state.diffs_pending_apply.is_empty() {
                std::mem::swap(
                    &mut diffs_pending_apply,
                    &mut editor_state.diffs_pending_apply,
                );
            }
        }

        // Apply the diffs to the world state
        for queued_diff in diffs_pending_apply {
            // Apply the diff to world state
            Self::apply_diff(
                world,
                resources,
                &queued_diff.diffs.apply_diff(),
                queued_diff.post_commit_selection,
            );

            // If commit is flagged, add an undo step will be added
            if queued_diff.commit_changes {
                let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
                editor_state.push_to_undo_queue(queued_diff.diffs);
            }
        }
    }

    fn clear_undo_history(&mut self) {
        self.undo_chain.clear();
        self.undo_chain_position = 0;
    }

    fn push_to_undo_queue(
        &mut self,
        diffs: TransactionDiffs,
    ) {
        // Drop everything that follows the current undo chain index
        self.undo_chain.truncate(self.undo_chain_position);

        // Push the given data onto the chain
        self.undo_chain.push_back(Arc::new(diffs));

        // We assume the caller has done whatever was needed
        self.undo_chain_position += 1;

        log::info!(
            "Pushed to undo queue, undo chain length: {} position: {}",
            self.undo_chain.len(),
            self.undo_chain_position
        );
    }

    fn undo(
        world: &mut World,
        resources: &Resources,
    ) {
        //TODO: Unclear what to do if there is an active transaction
        let diffs = {
            let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
            log::info!(
                "Going to undo, undo chain length: {} position: {}",
                editor_state.undo_chain.len(),
                editor_state.undo_chain_position
            );

            if editor_state.undo_chain_position > 0 {
                // reduce undo_index
                editor_state.undo_chain_position -= 1;

                // undo whatever is at self.undo_chain[self.undo_chain_index]
                Some(editor_state.undo_chain[editor_state.undo_chain_position].clone())
            } else {
                None
            }
        };

        if let Some(diffs) = diffs {
            Self::apply_diff(
                world,
                resources,
                &diffs.revert_diff(),
                PostCommitSelection::SelectAllInTransaction,
            );
        }
    }

    fn redo(
        world: &mut World,
        resources: &Resources,
    ) {
        //TODO: Unclear what to do if there is an active transaction
        let diffs = {
            let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
            log::info!(
                "Going to redo, undo chain length: {} position: {}",
                editor_state.undo_chain.len(),
                editor_state.undo_chain_position
            );

            if editor_state.undo_chain_position < editor_state.undo_chain.len() {
                // redo whatever is at self.undo_chain[self.undo_chain_index]
                let diffs = editor_state.undo_chain[editor_state.undo_chain_position].clone();

                // increase undo_index
                editor_state.undo_chain_position += 1;

                Some(diffs)
            } else {
                None
            }
        };

        if let Some(diffs) = diffs {
            Self::apply_diff(
                world,
                resources,
                &diffs.apply_diff(),
                PostCommitSelection::SelectAllInTransaction,
            );
        }
    }

    fn apply_diff(
        world: &mut World,
        resources: &Resources,
        diffs: &WorldDiff,
        post_commit_selection: PostCommitSelection,
    ) {
        let selected_uuids = {
            let mut selection_resource = resources.get_mut::<EditorSelectionResource>().unwrap();
            let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();

            // Clone the currently opened prefab Arc so we can refer back to it
            let mut opened_prefab = {
                if editor_state.opened_prefab.is_none() {
                    return;
                }

                editor_state.opened_prefab.as_ref().unwrap().clone()
            };

            // Get the UUIDs of all selected entities
            let selected_uuids = editor_state.get_selected_uuids(&mut *selection_resource, world);

            // Delete the old stuff from the world
            for x in opened_prefab.prefab_to_world_mappings.values() {
                world.delete(*x);
            }

            {
                let component_registry = resources.get::<ComponentRegistryResource>().unwrap();

                // Apply the diffs to the cooked data
                let mut universe = resources.get_mut::<UniverseResource>().unwrap();
                let new_cooked_prefab = Arc::new(legion_transaction::apply_diff_to_cooked_prefab(
                    &opened_prefab.cooked_prefab,
                    &universe.universe,
                    &diffs,
                    component_registry.components_by_uuid(),
                    &component_registry.copy_clone_impl(),
                ));

                let new_uncooked_prefab = Arc::new(legion_transaction::apply_diff_to_prefab(
                    &opened_prefab.uncooked_prefab,
                    &universe.universe,
                    &diffs,
                    &component_registry.components_by_uuid(),
                    &component_registry.copy_clone_impl(),
                ));

                // Update the opened prefab state
                let new_opened_prefab = OpenedPrefabState {
                    uuid: opened_prefab.uuid,
                    cooked_prefab: new_cooked_prefab,
                    prefab_handle: opened_prefab.prefab_handle.clone(),
                    uncooked_prefab: new_uncooked_prefab,
                    version: opened_prefab.version,
                    prefab_to_world_mappings: Default::default(), // These will get populated by reset()
                    world_to_prefab_mappings: Default::default(), // These will get populated by reset()
                };

                // Set opened_prefab (TODO: Probably better to pass new_opened_prefab in and let reset() assign to opened_prefab)
                editor_state.opened_prefab = Some(Arc::new(new_opened_prefab));
            }

            selected_uuids
        };

        // Spawn everything
        Self::reset(world, resources);

        match post_commit_selection {
            PostCommitSelection::SelectAllInTransaction => {
                let mut entity_uuids = HashSet::new();
                for d in diffs.entity_diffs() {
                    entity_uuids.insert(*d.entity_uuid());
                }

                for d in diffs.component_diffs() {
                    entity_uuids.insert(*d.entity_uuid());
                }

                let mut selection_resource =
                    resources.get_mut::<EditorSelectionResource>().unwrap();
                let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
                editor_state.restore_selected_uuids(&mut *selection_resource, world, &entity_uuids);
            }
            PostCommitSelection::KeepCurrentSelection => {
                let mut selection_resource =
                    resources.get_mut::<EditorSelectionResource>().unwrap();
                let mut editor_state = resources.get_mut::<EditorStateResource>().unwrap();
                editor_state.restore_selected_uuids(
                    &mut *selection_resource,
                    world,
                    &selected_uuids,
                );
            }
        }
    }

    fn save(
        &mut self,
        component_registry: &ComponentRegistry,
    ) {
        //
        // Check that a prefab is opened
        //
        if self.opened_prefab.is_none() {
            return;
        }

        let opened_prefab = self.opened_prefab.as_ref().unwrap();

        //
        // Persist the uncooked prefab to disk
        //
        let registered_components = component_registry.components_by_uuid();
        let prefab_serde_context = legion_prefab::PrefabSerdeContext {
            registered_components: registered_components.clone(),
        };

        let mut ron_ser = ron::ser::Serializer::new(Some(ron::ser::PrettyConfig::default()), true);
        let prefab_ser = legion_prefab::PrefabFormatSerializer::new(
            &prefab_serde_context,
            &opened_prefab.uncooked_prefab,
        );
        prefab_format::serialize(
            &mut ron_ser,
            &prefab_ser,
            opened_prefab.uncooked_prefab.prefab_id(),
        )
        .expect("failed to round-trip prefab");

        let output = ron_ser.into_output_string();
        log::trace!("Exporting prefab:");
        log::trace!("{}", output);

        std::fs::write("assets/demo_level.prefab", output).unwrap();
    }

    pub fn create_empty_transaction(
        &self,
        universe_resource: &UniverseResource,
        component_registry: &ComponentRegistry,
    ) -> Option<EditorTransaction> {
        if let Some(opened_prefab) = &self.opened_prefab {
            let mut tx_builder = TransactionBuilder::new();

            Some(EditorTransaction::new(
                tx_builder,
                &universe_resource.universe,
                &opened_prefab.cooked_prefab().world,
                component_registry,
            ))
        } else {
            None
        }
    }

    pub fn create_transaction_from_selected(
        &self,
        selection_resources: &EditorSelectionResource,
        universe_resource: &UniverseResource,
        component_registry: &ComponentRegistry,
    ) -> Option<EditorTransaction> {
        if selection_resources.selected_entities().is_empty() {
            return None;
        }

        if let Some(opened_prefab) = &self.opened_prefab {
            // Reverse the keys/values of the opened prefab map so we can efficiently look up the UUID of entities in the prefab
            use std::iter::FromIterator;
            let prefab_entity_to_uuid: HashMap<Entity, prefab_format::EntityUuid> =
                HashMap::from_iter(
                    opened_prefab
                        .cooked_prefab()
                        .entities
                        .iter()
                        .map(|(k, v)| (*v, *k)),
                );

            let mut tx_builder = TransactionBuilder::new();
            for world_entity in selection_resources.selected_entities() {
                if let Some(prefab_entity) =
                    opened_prefab.world_to_prefab_mappings().get(world_entity)
                {
                    if let Some(entity_uuid) = prefab_entity_to_uuid.get(prefab_entity) {
                        tx_builder = tx_builder.add_entity(*prefab_entity, *entity_uuid);
                    }
                }
            }

            Some(EditorTransaction::new(
                tx_builder,
                &universe_resource.universe,
                &opened_prefab.cooked_prefab().world,
                component_registry,
            ))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct EditorTransactionId(uuid::Uuid);

/// Wraps a transaction with an ID so that if downstream code interleaves multiple transactions,
/// we can detect this and commit the currently in-progress transaction. This lets us generate
/// undo/redo steps.
pub struct EditorTransaction {
    id: EditorTransactionId,
    transaction: legion_transaction::Transaction,
}

impl EditorTransaction {
    pub fn new(
        builder: TransactionBuilder,
        universe: &Universe,
        world: &World,
        component_registry: &ComponentRegistry,
    ) -> EditorTransaction {
        let id = EditorTransactionId(uuid::Uuid::new_v4());
        let transaction = builder.begin(universe, world, &component_registry.copy_clone_impl());

        EditorTransaction { id, transaction }
    }

    pub fn world(&self) -> &World {
        self.transaction.world()
    }

    pub fn world_mut(&mut self) -> &mut World {
        self.transaction.world_mut()
    }

    /// Writes data to the world without an undo step. The transaction can be cancelled to return
    /// the world to the state when the transaction began.
    pub fn update(
        &mut self,
        editor_state: &mut EditorStateResource,
        post_commit_selection: PostCommitSelection,
        component_registry: &ComponentRegistry,
    ) {
        log::info!("update transaction");
        self.do_update(
            editor_state,
            false,
            PostCommitSelection::KeepCurrentSelection,
            component_registry,
        );
    }

    /// Commits the transaction, writing an undo step
    pub fn commit(
        mut self,
        editor_state: &mut EditorStateResource,
        post_commit_selection: PostCommitSelection,
        component_registry: &ComponentRegistry,
    ) {
        log::info!("commit transaction");
        self.do_update(
            editor_state,
            true,
            post_commit_selection,
            component_registry,
        );
    }

    /// Reverts the changes that were made in this transaction without writing undo information
    pub fn cancel(
        &mut self,
        editor_state: &mut EditorStateResource,
        component_registry: &ComponentRegistry,
    ) {
        log::info!("cancel transaction");

        // Create diffs for this transaction
        let mut diffs = self
            .transaction
            .create_transaction_diffs(component_registry.components_by_uuid());

        // Reverse the apply/revert step, this ensures we do the revert instead of the apply
        diffs.reverse();

        // Apply the diffs, this is not a commit since we don't want this in the undo queue
        editor_state.enqueue_diffs(diffs, false, PostCommitSelection::KeepCurrentSelection);
    }

    fn do_update(
        &mut self,
        editor_state: &mut EditorStateResource,
        commit_changes: bool,
        post_commit_selection: PostCommitSelection,
        component_registry: &ComponentRegistry,
    ) {
        // If there is another transaction in progress, commit the old one.
        let commit_current_tx = match &editor_state.current_transaction_info {
            Some(info) => info.id != self.id,
            None => false,
        };

        if commit_current_tx {
            log::info!("commiting prior transaction");
            let mut current_transaction_info = None;
            std::mem::swap(
                &mut current_transaction_info,
                &mut editor_state.current_transaction_info,
            );
            editor_state.enqueue_diffs(
                current_transaction_info.unwrap().diffs,
                true,
                post_commit_selection,
            );
        }

        // Create diffs for this transaction
        let diffs = self
            .transaction
            .create_transaction_diffs(component_registry.components_by_uuid());

        // Update the current transaction info on the editor state. This is necessary book-keeping
        // to handle multiple transactions.
        if commit_changes {
            editor_state.current_transaction_info = None;
        } else {
            log::info!("saving transaction for future commit");
            editor_state.current_transaction_info = Some(CurrentTransactionInfo {
                id: self.id,
                diffs: diffs.clone(),
            });
        }

        // Apply the diffs, if commit_changes is true, an undo step will be added
        editor_state.enqueue_diffs(diffs, commit_changes, post_commit_selection);
    }
}
