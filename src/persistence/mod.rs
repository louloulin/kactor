mod event_store;
mod snapshot_store;
mod persistent_actor;
mod recovery;

pub use event_store::{EventStore, Event};
pub use snapshot_store::{SnapshotStore, Snapshot};
pub use persistent_actor::{PersistentActor, Persistence};
pub use recovery::{Recovery, RecoveryStrategy};

use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::{Actor, Context, Message}; 