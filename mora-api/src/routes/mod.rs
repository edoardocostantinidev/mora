use std::sync::{Arc, Mutex};

use mora_core::context::MoraContext;

pub mod channels;
pub mod events;
pub mod health;
pub mod queues;

type MutableMoraContext = Arc<Mutex<MoraContext>>;
