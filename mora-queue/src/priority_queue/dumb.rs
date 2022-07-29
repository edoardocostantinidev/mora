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

    fn peek(&self) -> Option<(K, V)> {
        let mut keys: Vec<K> = self.map.keys().cloned().collect::<Vec<K>>();
        keys.sort();
        let first_key: Option<K> = keys
            .iter()
            .take(1)
            .cloned()
            .collect::<Vec<K>>()
            .first()
            .cloned();
        first_key
            .map(|k| self.map.get_key_value(&k).unwrap())
            .map(|kv| (kv.0.clone(), kv.1.clone()))
    }
}
