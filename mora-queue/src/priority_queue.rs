use std::collections::HashMap;
use std::hash::Hash;

pub struct PriorityQueue<T, S>
where
    T: Clone + Eq + Hash + Ord,
    S: Clone,
{
    map: HashMap<T, S>,
}

impl<T, S> Default for PriorityQueue<T, S>
where
    T: Clone + Eq + Hash + Ord,
    S: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, S> PriorityQueue<T, S>
where
    T: Clone + Eq + Hash + Ord,
    S: Clone,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::<T, S>::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn enqueue(&mut self, key: T, value: S) -> Option<S> {
        self.map.insert(key, value)
    }

    pub fn take(&mut self, count: usize) -> Vec<S> {
        let mut keys: Vec<T> = self.map.keys().cloned().collect::<Vec<T>>();
        keys.sort();
        let values = keys
            .iter()
            .take(count)
            .map(|k| self.map.remove(k).unwrap())
            .collect::<Vec<S>>();

        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue_is_empty() {
        let pq = PriorityQueue::<u32, u32>::new();
        assert!(pq.is_empty())
    }

    #[test]
    fn new_queue_has_zero_elements() {
        let pq = PriorityQueue::<u32, u32>::new();
        assert_eq!(pq.len(), 0)
    }

    #[test]
    fn enqueue_adds_element_to_queue() {
        let mut pq = PriorityQueue::<u32, u32>::new();
        pq.enqueue(1, 1);
        assert_eq!(pq.len(), 1);
        assert!(!pq.is_empty());
    }

    #[test]
    fn take_elments_returns_elements_ordered_by_key() {
        let mut pq = PriorityQueue::<u32, u32>::new();
        pq.enqueue(2, 2);
        pq.enqueue(1, 1);
        pq.enqueue(3, 3);
        pq.enqueue(4, 3);
        let values: Vec<u32> = pq.take(3);

        assert_eq!(values.len(), 3);
        assert_eq!(values, [1, 2, 3]);
        assert_eq!(pq.len(), 1)
    }
}
