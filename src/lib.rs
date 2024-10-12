use serenity::prelude::*;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::SystemTime;

pub mod bot;
pub mod commands;
pub mod store;
pub mod tasks;
pub mod utils;

pub trait ContextData {
    fn data(&self) -> &Arc<RwLock<TypeMap>>;
}

impl ContextData for Context {
    fn data(&self) -> &Arc<RwLock<TypeMap>> {
        &self.data
    }
}

pub struct StartTime;
impl TypeMapKey for StartTime {
    type Value = SystemTime;
}

pub struct CleanupTaskCount;
impl TypeMapKey for CleanupTaskCount {
    type Value = AtomicUsize;
}

// Re-export commands
pub use commands::clean::clean as CLEAN_COMMAND;
pub use commands::status::status as STATUS_COMMAND;
