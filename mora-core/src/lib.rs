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

    pub fn add_queue(&mut self, queue_name: String) -> Result<(), String> {
        if let true = self.temporal_queues.contains_key(&queue_name) {
            return Err("queue already exists".to_owned());
        }

        self.temporal_queues.insert(queue_name, Default::default());
        Ok(())
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
        context.add_queue("test_queue".to_string())?;
        let queue = context.get_queue("test_queue").unwrap();
        queue.enqueue(1, "test".as_bytes().to_owned())?;
        assert_eq!(queue.len, 1);
        context.clear();
        assert!(context.get_queue("test_queue").is_none());
        Ok(())
    }

    #[test]
    fn add_queue_adds_queue() -> Result<(), String> {
        let mut context: MoraContext = Default::default();
        context.add_queue("test_queue_1".to_string())?;
        context.add_queue("test_queue_2".to_string())?;
        let queue = context.get_queue("test_queue_1");
        assert!(queue.is_some());
        let queue = context.get_queue("test_queue_2");
        assert!(queue.is_some());
        Ok(())
    }

    #[test]
    fn add_queue_doesnt_add_if_queue_already_exists() -> Result<(), String> {
        let mut context: MoraContext = Default::default();
        let result = context.add_queue("test_queue_1".to_string());
        assert!(result.is_ok());
        let result = context.add_queue("test_queue_1".to_string());
        assert!(result.is_err());
        Ok(())
    }
}
