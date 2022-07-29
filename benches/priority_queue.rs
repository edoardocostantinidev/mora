use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mora_queue::priority_queue::{
    dumb::DumbPriorityQueue, naive::NaivePriorityQueue, PriorityQueue,
};

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

fn naive_enqueue(c: &mut Criterion) {
    let mut pq = NaivePriorityQueue::<i32, i32>::default();
    let mut count = 0;
    c.bench_function("naive_enqueue", |b| {
        b.iter(|| {
            count = count + 1;
            pq.enqueue(black_box(count), black_box(-count))
        })
    });
}

fn naive_dequeue(c: &mut Criterion) {
    let mut pq = NaivePriorityQueue::<i32, i32>::default();

    (0..10000).for_each(|x| match x % 2 {
        1 => {
            pq.enqueue(-x, x);
        }
        0 => {
            pq.enqueue(x, x);
        }
        _ => {}
    });

    c.bench_function("naive_dequeue", |b| b.iter(|| pq.dequeue(black_box(100))));
}

criterion_group!(dumb_benches, dumb_enqueue, dumb_dequeue);
criterion_group!(naive_benches, naive_enqueue, naive_dequeue);

criterion_main!(dumb_benches, naive_benches);
