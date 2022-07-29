use mora_queue::temporal_queue::TemporalQueue;
use std::collections::hash_map::HashMap;
type Value = Vec<u8>;

#[derive(Debug)]
pub struct MoraContext {
    temporal_queues: HashMap<String, TemporalQueue<Value>>,
}

impl Default for MoraContext {
    fn default() -> Self {
        Self {
            temporal_queues: Default::default(),
        }
    }
}

impl MoraContext {
    pub fn clear(&mut self) {
        self.temporal_queues = Default::default()
    }

    pub fn add_queue(&mut self, arg: &str) -> Option<()> {
        self.temporal_queues
            .insert(arg.to_owned(), Default::default())
            .map(|_| ())
    }

    pub fn get_queue(&mut self, queue_name: &str) -> Option<&mut TemporalQueue<Value>> {
        self.temporal_queues.get_mut(queue_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clear_clears_context() -> Result<(), String> {
        let mut context: MoraContext = Default::default();
        context.add_queue("test_queue");
        let queue = context.get_queue("test_queue").unwrap();
        queue.enqueue(1, "test".as_bytes().to_owned())?;
        assert_eq!(queue.len, 1);
        context.clear();
        assert!(context.get_queue("test_queue").is_none());
        Ok(())
    }
}
