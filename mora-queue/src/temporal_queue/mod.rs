use mora_core::result::MoraError;

use crate::priority_queue::{naive::NaivePriorityQueue, PriorityQueue};

#[derive(Debug, Clone)]
pub struct TemporalQueue<V> {
    inner: NaivePriorityQueue<u128, V>,
    capacity: u128,
    pub len: u128,
}

impl<V> Default for TemporalQueue<V>
where
    V: Clone,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            len: 0,
            capacity: u128::MAX,
        }
    }
}

impl<V> TemporalQueue<V>
where
    V: Clone,
{
    pub fn new(capacity: u128) -> Self {
        Self {
            capacity,
            ..Default::default()
        }
    }
}

impl<V> TemporalQueue<V>
where
    V: Clone,
{
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Enqueues a value
    pub(crate) fn enqueue(&mut self, timestamp: u128, value: V) -> Result<(), MoraError> {
        match self.capacity {
            n if self.len == n => Err(MoraError::QueueFull),
            _ => {
                self.inner.enqueue(timestamp, value);
                self.len += 1;
                Ok(())
            }
        }
    }

    pub fn dequeue_until(&mut self, timestamp: u128, delete: bool) -> Vec<(u128, V)> {
        //todo: improve here using apposite data structure
        let mut values: Vec<(u128, V)> = vec![];
        // this is really bad
        self.inner
            .clone()
            .take_while(|(k, _)| *k <= timestamp)
            .for_each(|pair| {
                if delete {
                    self.len -= 1;
                    self.inner.dequeue(1);
                }
                values.push(pair);
            });

        values
    }
}

#[cfg(test)]
mod tests {
    use mora_core::result::MoraResult;

    use super::*;

    #[test]
    fn default_temporal_queue_is_empty() {
        assert!(TemporalQueue::<i32>::default().is_empty())
    }

    #[test]
    fn temporal_queue_should_enqueue_items_correctly() -> MoraResult<()> {
        let mut tq = TemporalQueue::<i32>::default();
        tq.enqueue(3, 3)?;
        tq.enqueue(4, 4)?;
        tq.enqueue(2, 2)?;
        tq.enqueue(1, 1)?;
        assert_eq!(tq.inner.dequeue(4), vec![1, 2, 3, 4]);
        Ok(())
    }

    #[test]
    fn temporal_queue_dequeue_until_dequeues_until_given_timestamp() -> MoraResult<()> {
        let mut tq = TemporalQueue::<i32>::default();
        tq.enqueue(1, 1)?;
        tq.enqueue(2, 2)?;
        tq.enqueue(3, 3)?;
        tq.enqueue(4, 4)?;
        let result = tq.dequeue_until(2, true);
        assert_eq!(result, vec![(1, 1), (2, 2)]);
        assert_eq!(tq.len, 2);
        Ok(())
    }
}
