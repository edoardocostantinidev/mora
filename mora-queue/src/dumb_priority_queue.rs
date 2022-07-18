use std::collections::HashMap;
use std::hash::Hash;

use crate::priority_queue::PriorityQueue;

pub struct DumbPriorityQueue<K, V>
where
    K: Clone + Eq + Hash + Ord,
    V: Clone,
{
    map: HashMap<K, V>,
}

impl<K, V> Default for DumbPriorityQueue<K, V>
where
    K: Clone + Eq + Hash + Ord,
    V: Clone,
{
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

impl<K, V> PriorityQueue<K, V> for DumbPriorityQueue<K, V>
where
    K: Clone + Eq + Hash + Ord,
    V: Clone,
{
    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    #[inline]
    fn enqueue(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }

    #[inline]
    fn dequeue(&mut self, count: usize) -> Vec<V> {
        let mut keys: Vec<K> = self.map.keys().cloned().collect::<Vec<K>>();
        keys.sort();
        keys.iter()
            .take(count)
            .map(|k| self.map.remove(k).unwrap())
            .collect::<Vec<V>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue_is_empty() {
        let pq = DumbPriorityQueue::<u32, u32>::default();
        assert!(pq.is_empty())
    }

    #[test]
    fn new_queue_has_zero_elements() {
        let pq = DumbPriorityQueue::<u32, u32>::default();
        assert_eq!(pq.len(), 0)
    }

    #[test]
    fn enqueue_adds_element_to_queue() {
        let mut pq = DumbPriorityQueue::<u32, u32>::default();
        pq.enqueue(1, 1);
        assert_eq!(pq.len(), 1);
        assert!(!pq.is_empty());
    }

    #[test]
    fn take_elments_returns_elements_ordered_by_key() {
        let mut pq = DumbPriorityQueue::<u32, u32>::default();
        pq.enqueue(4, 3);
        pq.enqueue(2, 2);
        pq.enqueue(1, 1);
        pq.enqueue(3, 3);
        let values: Vec<u32> = pq.dequeue(3);

        assert_eq!(values.len(), 3);
        assert_eq!(values, [1, 2, 3]);
        assert_eq!(pq.len(), 1)
    }
}
