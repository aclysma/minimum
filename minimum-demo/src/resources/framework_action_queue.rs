use std::path::PathBuf;
use std::collections::VecDeque;
use minimum::ResourceMap;
use crate::PlayMode;
use crate::framework;
use crate::resources;
use minimum::Component;
use crate::components;

pub struct FrameworkActionQueue {
    queue: VecDeque<Box<dyn FnOnce(&ResourceMap) + Send + Sync>>
}

impl FrameworkActionQueue {
    pub fn new() -> Self {
        FrameworkActionQueue {
            queue: VecDeque::new()
        }
    }

    //
    // Load level from file
    //
    pub fn enqueue_load_level(&mut self, path: PathBuf) {
        self.queue.push_back(Box::new(move |_resource_map| {
            println!("load level {:?}", path);
        }));
    }

    //
    // Save level to file
    //
    pub fn enqueue_save_level(&mut self, path: PathBuf) {
        self.queue.push_back(Box::new(move |resource_map| {
            println!("save level {:?}", path);
            let persist_registry = resource_map.fetch::<framework::persist::PersistRegistry>();
            persist_registry.save(resource_map);
        }));
    }

    //
    // change_play_mode
    //
    pub fn enqueue_change_play_mode(&mut self, new_play_mode: PlayMode) {
        self.queue.push_back(Box::new(move |resource_map| {
            println!("change_play_mode {:?}", new_play_mode);
            // Clear playmode flags
            let mut dispatch_control = resource_map.fetch_mut::<minimum::DispatchControl>();
            *dispatch_control.next_frame_context_flags_mut() &=
                !(crate::context_flags::PLAYMODE_SYSTEM
                    | crate::context_flags::PLAYMODE_PAUSED
                    | crate::context_flags::PLAYMODE_PLAYING);

            // Set the appropriate ones
            match new_play_mode {
                PlayMode::System => {
                    *dispatch_control.next_frame_context_flags_mut() |=
                        crate::context_flags::PLAYMODE_SYSTEM;
                }
                PlayMode::Paused => {
                    *dispatch_control.next_frame_context_flags_mut() |=
                        crate::context_flags::PLAYMODE_SYSTEM
                            | crate::context_flags::PLAYMODE_PAUSED;
                }
                PlayMode::Playing => {
                    *dispatch_control.next_frame_context_flags_mut() |=
                        crate::context_flags::PLAYMODE_SYSTEM
                            | crate::context_flags::PLAYMODE_PAUSED
                            | crate::context_flags::PLAYMODE_PLAYING;

                    let mut editor_ui_state = resource_map.fetch_mut::<resources::EditorUiState>();
                    editor_ui_state.set_inspector_tab = Some(framework::inspect::InspectorTab::Runtime);
                }
            }
        }));
    }

    //
    // Reset level
    //
    pub fn enqueue_reset_level(&mut self) {
        self.queue.push_back(Box::new(move |resource_map| {
            println!("enqueue_reset_level");
            // Collect all the data needed to re-create the persistent entities
            let prototypes = {
                let mut prototypes = vec![];

                // Every persistent entity will have a component with the components that created it
                let persistent_entity_components = resource_map.fetch::<<components::PersistentEntityComponent as Component>::Storage>();
                for persistent_entity_component in persistent_entity_components.iter_values() {
                    prototypes.push(persistent_entity_component.entity_prototype().clone());
                }

                prototypes
            };

            let mut entity_set = resource_map.fetch_mut::<minimum::EntitySet>();
            entity_set.clear(resource_map);

            let mut entity_factory = resource_map.fetch_mut::<minimum::EntityFactory>();
            for prototype in prototypes {
                entity_factory.enqueue_create(Box::new(prototype));
            }

            let mut editor_ui_state = resource_map.fetch_mut::<resources::EditorUiState>();
            editor_ui_state.set_inspector_tab = Some(framework::inspect::InspectorTab::Persistent);
        }));
    }

    //
    // Terminate process
    //
    pub fn enqueue_terminate_process(&mut self) {
        self.queue.push_back(Box::new(move |resource_map| {
            println!("enqueue_terminate_process");
            let mut dispatch_control = resource_map.fetch_mut::<minimum::DispatchControl>();
            dispatch_control.end_game_loop();
        }));
    }


    pub fn process_queue(&mut self, resource_map: &ResourceMap) {
        for action in self.queue.drain(..) {
            (action)(resource_map);
        }
    }
}
