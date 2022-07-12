use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mora_queue::priority_queue::PriorityQueue;

fn enqueue(c: &mut Criterion) {
    let mut pq = PriorityQueue::<i32, i32>::new();
    c.bench_function("enqueue", |b| {
        b.iter(|| pq.enqueue(black_box(1), black_box(2)))
    });
}

fn take(c: &mut Criterion) {
    let mut pq = PriorityQueue::<i32, i32>::new();

    (0..10000).for_each(|x| match x % 2 {
        1 => {
            pq.enqueue(-x, x);
        }
        0 => {
            pq.enqueue(x, x);
        }
        _ => {}
    });

    c.bench_function("take", |b| b.iter(|| pq.take(black_box(100))));
}

criterion_group!(queue_benches, enqueue, take);
criterion_main!(queue_benches);
