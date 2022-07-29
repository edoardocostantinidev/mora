use std::{cmp::min, collections::VecDeque};

use super::PriorityQueue;

#[derive(Clone, Debug)]
struct Node<K, V> {
    key: K,
    value: V,
}
pub struct NaivePriorityQueue<K, V> {
    items: VecDeque<Node<K, V>>,
}

impl<K, V> Default for NaivePriorityQueue<K, V>
where
    K: Clone + Eq + Ord,
    V: Clone,
{
    fn default() -> Self {
        Self {
            items: Default::default(),
        }
    }
}

impl<K: Clone + Ord, V: Clone> PriorityQueue<K, V> for NaivePriorityQueue<K, V> {
    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn enqueue(&mut self, key: K, value: V) -> Option<V> {
        let mut index: Option<usize> = None;

        if self.is_empty() {
            self.items.insert(
                0,
                Node {
                    key,
                    value: value.clone(),
                },
            );
            return Some(value);
        }

        for (i, v) in self.items.iter().enumerate() {
            if key < v.key {
                index = Some(i);
                break;
            }
        }

        let index = match index {
            None => self.items.len(),
            Some(i) => i,
        };

        self.items.insert(
            index,
            Node {
                key,
                value: value.clone(),
            },
        );
        Some(value)
    }

    fn dequeue(&mut self, count: usize) -> Vec<V> {
        let mut items: Vec<V> = Default::default();
        let range = 0..min(count, self.items.len());

        for _ in range {
            let v: V = self.items.pop_front().unwrap().value;
            items.push(v);
        }

        items
    }

    fn peek(&self) -> Option<(K, V)> {
        self.items.get(0).map(|n| (n.key.clone(), n.value.clone()))
    }
}
