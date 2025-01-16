use std::ops::Range;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use rand::seq::SliceRandom;
use rand::thread_rng;
use urcu::prelude::*;

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

    fn run(self) -> u128 {
        let mut context = RcuDefaultFlavor::rcu_context_builder()
            .with_read_context()
            .register_thread()
            .unwrap();

        let mut node_count = 0u128;

        while !self.exit_signal.load(Ordering::Acquire) {
            let mut keyset = self.keyset.clone().collect::<Vec<_>>();
            keyset.shuffle(&mut thread_rng());

            node_count += self.publish(&keyset, &mut context);
        }

        println!(
            "published {} nodes in [{}, {}]",
            node_count, self.keyset.start, self.keyset.end
        );

        self.publisher_count.fetch_sub(1, Ordering::Release);

        node_count
    }

    fn publish<C>(&self, keyset: &[u32], context: &mut C) -> u128
    where
        C: RcuReadContext<Flavor = RcuDefaultFlavor>,
    {
        let mut node_inserted = 0u128;

        for key in keyset {
            context.rcu_quiescent_state();

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

    fn run(self) -> u128 {
        let mut context = RcuDefaultFlavor::rcu_context_builder()
            .with_read_context()
            .with_defer_context()
            .register_thread()
            .unwrap();

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

        node_count
    }

    fn consume<C>(&self, keyset: &[u32], context: &mut C) -> u128
    where
        C: RcuReadContext<Flavor = RcuDefaultFlavor>
            + RcuDeferContext<Flavor = RcuDefaultFlavor>
            + 'static,
    {
        let mut node_removed = 0u128;

        for key in keyset {
            context.rcu_quiescent_state();

            let guard = context.rcu_read_lock();
            let item = self.map.remove(key, &guard);
            if let Some(ref item) = item {
                let got = item.value();
                let expected = key_to_value(*item.key());
                if *got != expected {
                    log::error!("map[{}] = {} != {}", got, item.key(), expected);
                }

                node_removed += 1;
            }

            drop(guard);

            match node_removed % 3 {
                0 => item.safe_cleanup(),
                1 => item.call_cleanup(context),
                2 => item.defer_cleanup(context),
                _ => panic!("unexpected"),
            }
        }

        node_removed
    }
}

/// Run a RCU list stress test using multiple threads.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of publisher threads.
    #[arg(short, long, default_value = "4")]
    publishers: u32,

    /// Number of consumer threads.
    #[arg(short, long, default_value = "4")]
    consumers: u32,

    /// Duration of the test.
    #[arg(short, long, default_value = "5s", value_parser = humantime::parse_duration)]
    duration: Duration,
}

struct ExitHandler(Receiver<()>);

impl ExitHandler {
    fn configure() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        ctrlc::set_handler(move || {
            println!();
            if tx.send(()).is_err() {
                log::error!("failed to send Ctrl+C signal");
            }
        })
        .expect("Error setting Ctrl-C handler");

        Self(rx)
    }

    fn wait_for(&self, duration: Duration) {
        if duration.is_zero() {
            if self.0.recv().is_err() {
                log::error!("Ctrl+C handler unexpectedly disconnected");
            }
        } else if let Err(RecvTimeoutError::Disconnected) = self.0.recv_timeout(duration) {
            log::error!("Ctrl+C handler unexpectedly disconnected");
        }
    }
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    std::thread::scope(|scope| {
        let exit = Arc::new(AtomicBool::new(false));
        let exit_handler = ExitHandler::configure();
        let map = RcuHashMap::<u32, u64>::new().unwrap();
        let mut ranges = RangeDistributor::new();

        let publisher_count = Arc::new(AtomicUsize::new(0));
        let publishers = (0..args.publishers)
            .map(|_| {
                let thread = PublisherThread::new(&exit, &publisher_count, ranges.get(), &map);
                scope.spawn(move || thread.run())
            })
            .collect::<Vec<_>>();

        ranges.reset();

        let consumers = (0..args.consumers)
            .map(|_| {
                let thread = ConsumerThread::new(&publisher_count, ranges.get(), &map);
                scope.spawn(move || thread.run())
            })
            .collect::<Vec<_>>();

        exit_handler.wait_for(args.duration);
        exit.store(true, Ordering::Release);

        let published_nodes = publishers
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .sum::<u128>();

        println!("published a total of {published_nodes} nodes");

        let consumed_nodes = consumers
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .sum::<u128>();

        println!("consumed a total of {consumed_nodes} nodes");

        assert_eq!(published_nodes, consumed_nodes);
    });
}

struct RangeDistributor {
    next: usize,
    ranges: Vec<Range<u32>>,
}

impl RangeDistributor {
    fn new() -> Self {
        Self {
            next: 0,
            ranges: vec![
                0..20000,
                0..10000,
                5000..15000,
                10000..20000,
                0..10000,
                5000..15000,
                10000..20000,
            ],
        }
    }

    fn get(&mut self) -> Range<u32> {
        let range = self.ranges[self.next].clone();
        self.next = (self.next + 1) % self.ranges.len();
        range
    }

    fn reset(&mut self) {
        self.next = 0;
    }
}
