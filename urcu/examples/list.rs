use std::ops::Deref;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use urcu::prelude::*;

struct ReaderThread {
    publisher_count: Arc<AtomicUsize>,
    list: Arc<RcuList<u32>>,
}

impl ReaderThread {
    fn new(publisher_count: &Arc<AtomicUsize>, list: &Arc<RcuList<u32>>) -> Self {
        Self {
            publisher_count: publisher_count.clone(),
            list: list.clone(),
        }
    }

    fn run(self) {
        let context: RcuDefaultContext = RcuDefaultContext::rcu_register().unwrap();

        let mut node_count = 0u128;
        let mut total_sum = 0u128;

        loop {
            if self.list.is_empty() {
                if self.publisher_count.load(Ordering::Acquire) == 0 {
                    break;
                }
            }

            let guard = context.rcu_read_lock();

            for value in self.list.iter_forward(&guard).peekable() {
                node_count += 1;
                total_sum += u128::from(*value);
            }

            drop(guard);
        }

        println!(
            "read {} nodes with a total sum of {}",
            node_count, total_sum
        );
    }
}

struct PublisherThread {
    exit_signal: Arc<AtomicBool>,
    publisher_count: Arc<AtomicUsize>,
    list: Arc<RcuList<u32>>,
}

impl PublisherThread {
    fn new(
        exit_signal: &Arc<AtomicBool>,
        publisher_count: &Arc<AtomicUsize>,
        list: &Arc<RcuList<u32>>,
    ) -> Self {
        publisher_count.fetch_add(1, Ordering::Release);

        Self {
            exit_signal: exit_signal.clone(),
            publisher_count: publisher_count.clone(),
            list: list.clone(),
        }
    }

    fn run(self) -> (u128, u128) {
        let mut node_count = 0;
        let mut total_sum = 0u128;
        let mut value = 0;

        while !self.exit_signal.load(Ordering::Acquire) {
            self.list.push_back(value).unwrap();
            self.list.push_front(value).unwrap();

            node_count += 2;
            total_sum += 2 * u128::from(value);
            value = (value + 1) % 1000;
        }

        self.publisher_count.fetch_sub(1, Ordering::Release);

        println!(
            "published {} nodes with a total sum of {}",
            node_count, total_sum
        );

        (node_count, total_sum)
    }
}

struct ConsumerThread {
    publisher_count: Arc<AtomicUsize>,
    list: Arc<RcuList<u32>>,
}

impl ConsumerThread {
    fn new(publisher_count: &Arc<AtomicUsize>, list: &Arc<RcuList<u32>>) -> Self {
        Self {
            publisher_count: publisher_count.clone(),
            list: list.clone(),
        }
    }

    fn run(self) -> (u128, u128) {
        let mut context = RcuDefaultContext::rcu_register().unwrap();

        let mut node_count = 0;
        let mut total_sum = 0u128;

        loop {
            let value = self.list.pop_back().unwrap();

            if let Some(value) = &value {
                node_count += 1;
                total_sum += u128::from(*value.deref());
            } else if self.publisher_count.load(Ordering::Acquire) == 0 {
                break;
            }

            match node_count % 3 {
                0 => value.safe_cleanup(),
                1 => value.call_cleanup(&context),
                2 => value.defer_cleanup(&mut context),
                _ => panic!("unexpected"),
            }
        }

        println!(
            "consumed {} nodes with a total sum of {}",
            node_count, total_sum
        );

        (node_count, total_sum)
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

    /// Number of reader threads.
    #[arg(short, long, default_value = "2")]
    readers: u32,

    /// Duration of the test.
    #[arg(short, long, default_value = "5s", value_parser = humantime::parse_duration)]
    duration: Duration,
}

struct ExitHandler(Receiver<()>);

impl ExitHandler {
    fn configure() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        ctrlc::set_handler(move || {
            println!("");
            if let Err(_) = tx.send(()) {
                log::error!("failed to send Ctrl+C signal");
            }
        })
        .expect("Error setting Ctrl-C handler");

        Self(rx)
    }

    fn wait_for(&self, duration: Duration) {
        if duration.is_zero() {
            if let Err(_) = self.0.recv() {
                log::error!("Ctrl+C handler unexpectedly disconnected");
            }
        } else {
            if let Err(RecvTimeoutError::Disconnected) = self.0.recv_timeout(duration) {
                log::error!("Ctrl+C handler unexpectedly disconnected");
            }
        }
    }
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    std::thread::scope(|scope| {
        let exit = Arc::new(AtomicBool::new(false));
        let exit_handler = ExitHandler::configure();
        let list = RcuList::<u32>::new();

        let publisher_count = Arc::new(AtomicUsize::new(0));
        let publishers = (0..args.publishers)
            .map(|_| {
                let thread = PublisherThread::new(&exit.clone(), &publisher_count, &list);
                scope.spawn(move || thread.run())
            })
            .collect::<Vec<_>>();

        let consumers = (0..args.consumers)
            .map(|_| {
                let thread = ConsumerThread::new(&publisher_count, &list);
                scope.spawn(move || thread.run())
            })
            .collect::<Vec<_>>();

        (0..args.readers).for_each(|_| {
            let thread = ReaderThread::new(&publisher_count, &list);
            scope.spawn(move || thread.run());
        });

        exit_handler.wait_for(args.duration);
        exit.store(true, Ordering::Release);

        let (published_nodes, published_total) = publishers
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .fold((0, 0), |(acc_nodes, acc_total), (nodes, total)| {
                (acc_nodes + nodes, acc_total + total)
            });

        println!(
            "published a total of {} nodes with a total sum of {}",
            published_nodes, published_total
        );

        let (consumed_nodes, consumed_total) = consumers
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .fold((0, 0), |(acc_nodes, acc_total), (nodes, total)| {
                (acc_nodes + nodes, acc_total + total)
            });

        println!(
            "consumed a total of {} nodes with a total sum of {}",
            consumed_nodes, consumed_total
        );

        assert_eq!(published_nodes, consumed_nodes);
        assert_eq!(published_total, consumed_total);
    });
}
