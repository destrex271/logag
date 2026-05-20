use crate::shared_log::traits::Event;

pub trait StorageEngine {
    fn store_event(&self, event: &dyn Event);
    fn get_events(&self, session_id: &str) -> impl Iterator<Item = dyn Event + 'static>;
}
