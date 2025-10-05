pub mod dumb;
pub mod naive;

// WARNING: implementations right now are virtually shit, they should be refactored to use a more efficient data structure
pub trait PriorityQueue<K, V>: Iterator<Item = (K, V)> {
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn enqueue(&mut self, key: K, value: V) -> Option<V>;
    fn dequeue(&mut self, count: usize) -> Vec<V>;
    fn peek(&self) -> Option<(K, V)>;
}

#[cfg(test)]
macro_rules! priority_queue_tests {
    ($($name:ident: $type:ty,)*) => {
    $(
        mod $name {
            use crate::priority_queue::PriorityQueue;

    #[test]
    fn new_queue_is_empty() {
        let pq = <$type>::default();
        assert!(pq.is_empty())
    }

    #[test]
    fn new_queue_has_zero_elements() {
        let pq = <$type>::default();
        assert_eq!(pq.len(), 0)
    }

    #[test]
    fn enqueue_adds_element_to_queue() {
        let mut pq = <$type>::default();
        pq.enqueue(1, 1);
        assert_eq!(pq.len(), 1);
        assert!(!pq.is_empty());
    }

    #[test]
    fn take_elments_returns_elements_ordered_by_key() {
        let mut pq = <$type>::default();
        pq.enqueue(4, 3);
        pq.enqueue(2, 2);
        pq.enqueue(1, 1);
        pq.enqueue(3, 3);
        let values: Vec<u32> = pq.dequeue(3);

        assert_eq!(values.len(), 3);
        assert_eq!(values, [1, 2, 3]);
        assert_eq!(pq.len(), 1)
        }

        #[test]
        fn peek_returns_first_item(){
            let mut pq = <$type>::default();
            pq.enqueue(4, 3);
            pq.enqueue(2, 2);
            pq.enqueue(1, 1);
            pq.enqueue(3, 3);
            let value= pq.peek();
            assert_eq!(value, Some((1,1)));
        }

        #[test]
        fn peek_on_empty_queue_return_none(){
            let pq = <$type>::default();
            let value= pq.peek();
            assert_eq!(value, None);
        }
    }
    )*
    }
}

#[cfg(test)]
mod tests {
    priority_queue_tests! {
        dumb_priority_queue: super::super::dumb::DumbPriorityQueue::<u32,u32>,
        naive_priority_queue: super::super::naive::NaivePriorityQueue<u32,u32>,
    }
}
