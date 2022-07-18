use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mora_queue::{dumb_priority_queue::DumbPriorityQueue, priority_queue::PriorityQueue};

fn dumb_enqueue(c: &mut Criterion) {
    let mut pq = DumbPriorityQueue::<i32, i32>::default();
    let mut count = 0;
    c.bench_function("dumb_enqueue", |b| {
        b.iter(|| {
            count = count + 1;
            pq.enqueue(black_box(count), black_box(-count))
        })
    });
}

fn dumb_dequeue(c: &mut Criterion) {
    let mut pq = DumbPriorityQueue::<i32, i32>::default();

    (0..10000).for_each(|x| match x % 2 {
        1 => {
            pq.enqueue(-x, x);
        }
        0 => {
            pq.enqueue(x, x);
        }
        _ => {}
    });

    c.bench_function("dumb_dequeue", |b| b.iter(|| pq.dequeue(black_box(100))));
}

criterion_group!(queue_benches, dumb_enqueue, dumb_dequeue);
criterion_main!(queue_benches);
