pub struct QueueService;

pub struct Queue {
    pub name: String,
}

impl Queue {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
impl QueueService {
    pub fn get_queues(&self, _context: &MoraContext) -> MoraResult<Vec<Queue>> {
        Ok(vec![])
    }
}

#[cfg(test)]
use mockall::predicate::*;
use mora_core::{context::MoraContext, result::MoraResult};

#[cfg(test)]
mockall::mock! {
    pub QueueService {
        pub fn get_queues(&self, context: &MoraContext) -> MoraResult<Vec<Queue>>;
    }
}
