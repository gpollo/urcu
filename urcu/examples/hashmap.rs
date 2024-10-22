use std::ops::Range;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::thread_rng;
use urcu::flavor::RcuContextMemb;
use urcu::{RcuContext, RcuHashMap, RcuReadContext, RcuRef};

fn key_to_value(key: u32) -> u64 {
    (key * (key + 1337)) as u64
}

struct PublisherThread {
    exit_signal: Arc<AtomicBool>,
    publisher_count: Arc<AtomicUsize>,
    keyset: Range<u32>,
    map: Arc<RcuHashMap<u32, u64>>,
}

impl PublisherThread {
    fn new(
        exit_signal: &Arc<AtomicBool>,
        publisher_count: &Arc<AtomicUsize>,
        keyset: Range<u32>,
        map: &Arc<RcuHashMap<u32, u64>>,
    ) -> Self {
        publisher_count.fetch_add(1, Ordering::Release);

        Self {
            exit_signal: exit_signal.clone(),
            publisher_count: publisher_count.clone(),
            keyset,
            map: map.clone(),
        }
    }

    fn run(self) {
        let context = RcuContextMemb::rcu_register().unwrap();
        let mut node_count = 0u128;

        while !self.exit_signal.load(Ordering::Acquire) {
            let mut keyset = self.keyset.clone().collect::<Vec<_>>();
            keyset.shuffle(&mut thread_rng());

            node_count += self.publish(&keyset, &context);
        }

        println!(
            "published {} nodes in [{}, {}]",
            node_count, self.keyset.start, self.keyset.end
        );

        self.publisher_count.fetch_sub(1, Ordering::Release);
    }

    fn publish(&self, keyset: &[u32], context: &RcuContextMemb) -> u128 {
        let mut node_inserted = 0u128;

        for key in keyset {
            let guard = context.rcu_read_lock();
            let item = self.map.insert(*key, key_to_value(*key), &guard);
            if item.is_none() {
                node_inserted += 1;
            }

            drop(guard);
        }

        node_inserted
    }
}

struct ConsumerThread {
    publisher_count: Arc<AtomicUsize>,
    keyset: Range<u32>,
    map: Arc<RcuHashMap<u32, u64>>,
}

impl ConsumerThread {
    fn new(
        publisher_count: &Arc<AtomicUsize>,
        keyset: Range<u32>,
        map: &Arc<RcuHashMap<u32, u64>>,
    ) -> Self {
        Self {
            publisher_count: publisher_count.clone(),
            keyset,
            map: map.clone(),
        }
    }

    fn run(self) {
        let mut context = RcuContextMemb::rcu_register().unwrap();
        let mut node_count = 0u128;

        loop {
            let mut keyset = self.keyset.clone().collect::<Vec<_>>();
            keyset.shuffle(&mut thread_rng());

            let node_removed = self.consume(&keyset, &mut context);
            if node_removed == 0 && self.publisher_count.load(Ordering::Acquire) == 0 {
                break;
            }

            node_count += node_removed;
        }

        println!(
            "consumed {} nodes in [{}, {}]",
            node_count, self.keyset.start, self.keyset.end
        );
    }

    fn consume(&self, keyset: &[u32], context: &mut RcuContextMemb) -> u128 {
        let mut node_removed = 0u128;

        for key in keyset {
            let guard = context.rcu_read_lock();
            let item = self.map.remove(&key, &guard);
            if let Some(ref item) = item {
                let got = item.value();
                let expected = key_to_value(*item.key());
                if *got != expected {
                    log::error!("map[{}] = {} != {}", got, item.key(), expected);
                }

                node_removed += 1;
            }

            drop(guard);
            item.defer_cleanup(context);
        }

        node_removed
    }
}

fn main() {
    let map = RcuHashMap::<u32, u64>::new().unwrap();
    let exit = Arc::new(AtomicBool::new(false));
    let exit_signal = exit.clone();
    let publisher_count = Arc::new(AtomicUsize::new(0));

    ctrlc::set_handler(move || {
        println!("");
        exit.store(true, Ordering::Release);
    })
    .expect("Error setting Ctrl-C handler");

    std::thread::scope(|scope| {
        let thread = PublisherThread::new(&exit_signal, &publisher_count, 0..10000, &map);
        scope.spawn(move || thread.run());

        let thread = PublisherThread::new(&exit_signal, &publisher_count, 5000..15000, &map);
        scope.spawn(move || thread.run());

        let thread = PublisherThread::new(&exit_signal, &publisher_count, 10000..20000, &map);
        scope.spawn(move || thread.run());

        let thread = PublisherThread::new(&exit_signal, &publisher_count, 0..10000, &map);
        scope.spawn(move || thread.run());

        let thread = PublisherThread::new(&exit_signal, &publisher_count, 5000..15000, &map);
        scope.spawn(move || thread.run());

        let thread = PublisherThread::new(&exit_signal, &publisher_count, 10000..20000, &map);
        scope.spawn(move || thread.run());

        let thread = ConsumerThread::new(&publisher_count, 0..10000, &map);
        scope.spawn(move || thread.run());

        let thread = ConsumerThread::new(&publisher_count, 5000..15000, &map);
        scope.spawn(move || thread.run());

        let thread = ConsumerThread::new(&publisher_count, 10000..20000, &map);
        scope.spawn(move || thread.run());

        let thread = ConsumerThread::new(&publisher_count, 0..10000, &map);
        scope.spawn(move || thread.run());

        let thread = ConsumerThread::new(&publisher_count, 5000..15000, &map);
        scope.spawn(move || thread.run());

        let thread = ConsumerThread::new(&publisher_count, 10000..20000, &map);
        scope.spawn(move || thread.run());
    });
}
