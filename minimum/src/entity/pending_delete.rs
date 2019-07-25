
//TODO: Use this rather than the queue in entity_set

use minimum::component::SlabComponentStorage;

#[derive(Debug)]
pub struct PendingDeleteComponent {

}

impl PendingDeleteComponent {
    pub fn new() -> Self {
        PendingDeleteComponent {

        }
    }
}

impl minimum::Component for PendingDeleteComponent {
    type Storage = SlabComponentStorage<PendingDeleteComponent>;
}
