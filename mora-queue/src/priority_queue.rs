pub trait PriorityQueue<K, V> {
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn enqueue(&mut self, key: K, value: V) -> Option<V>;
    fn dequeue(&mut self, count: usize) -> Vec<V>;
}
