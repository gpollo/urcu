use std::ops::Deref;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use urcu::context::RcuContextMemb;
use urcu::{RcuContext, RcuList, RcuReadContext, RcuRef};

struct ReaderThread {
    publisher_count: Arc<AtomicUsize>,
    list: Arc<RcuList<u32, RcuContextMemb>>,
}

impl ReaderThread {
    fn new(publisher_count: &Arc<AtomicUsize>, list: &Arc<RcuList<u32, RcuContextMemb>>) -> Self {
        Self {
            publisher_count: publisher_count.clone(),
            list: list.clone(),
        }
    }

    fn run(self) {
        let context = RcuContextMemb::rcu_register().unwrap();

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
    list: Arc<RcuList<u32, RcuContextMemb>>,
}

impl PublisherThread {
    fn new(
        exit_signal: &Arc<AtomicBool>,
        publisher_count: &Arc<AtomicUsize>,
        list: &Arc<RcuList<u32, RcuContextMemb>>,
    ) -> Self {
        publisher_count.fetch_add(1, Ordering::Release);

        Self {
            exit_signal: exit_signal.clone(),
            publisher_count: publisher_count.clone(),
            list: list.clone(),
        }
    }

    fn run(self) {
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
    }
}

struct ConsumerThread {
    publisher_count: Arc<AtomicUsize>,
    list: Arc<RcuList<u32, RcuContextMemb>>,
}

impl ConsumerThread {
    fn new(publisher_count: &Arc<AtomicUsize>, list: &Arc<RcuList<u32, RcuContextMemb>>) -> Self {
        Self {
            publisher_count: publisher_count.clone(),
            list: list.clone(),
        }
    }

    fn run(self) {
        let mut context = RcuContextMemb::rcu_register().unwrap();

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

            value.defer_cleanup(&mut context);
        }

        println!(
            "consumed {} nodes with a total sum of {}",
            node_count, total_sum
        );
    }
}

fn main() {
    let list = RcuList::<u32, RcuContextMemb>::new();
    let exit = Arc::new(AtomicBool::new(false));
    let exit_signal = exit.clone();
    let publisher_count = Arc::new(AtomicUsize::new(0));

    ctrlc::set_handler(move || {
        println!("");
        exit.store(true, Ordering::Release);
    })
    .expect("Error setting Ctrl-C handler");

    std::thread::scope(|scope| {
        let thread = PublisherThread::new(&exit_signal, &publisher_count, &list);
        scope.spawn(move || thread.run());

        let thread = ConsumerThread::new(&publisher_count, &list);
        scope.spawn(move || thread.run());

        let thread = ReaderThread::new(&publisher_count, &list);
        scope.spawn(move || thread.run());

        let thread = ReaderThread::new(&publisher_count, &list);
        scope.spawn(move || thread.run());

        let thread = ReaderThread::new(&publisher_count, &list);
        scope.spawn(move || thread.run());

        let thread = ReaderThread::new(&publisher_count, &list);
        scope.spawn(move || thread.run());

        let thread = ReaderThread::new(&publisher_count, &list);
        scope.spawn(move || thread.run());

        let thread = ReaderThread::new(&publisher_count, &list);
        scope.spawn(move || thread.run());

        let thread = ReaderThread::new(&publisher_count, &list);
        scope.spawn(move || thread.run());

        let thread = PublisherThread::new(&exit_signal, &publisher_count, &list);
        scope.spawn(move || thread.run());
    });
}
