use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use urcu::flavor::RcuContextMemb;
use urcu::linked_list::RcuList;
use urcu::{rcu_take_ownership, RcuContext};

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
        let context = RcuContextMemb::new().unwrap();

        let mut node_count = 0;
        let mut total_sum = 0;

        loop {
            let guard = context.rcu_read_lock();
            let reader = self.list.reader(&guard);
            let mut iterator = reader.iter_forward().peekable();

            if iterator.peek().is_none() {
                if self.publisher_count.load(Ordering::Relaxed) == 0 {
                    break;
                }
            }

            for value in reader.iter_forward() {
                node_count += 1;
                total_sum += value;
            }
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
        let mut total_sum = 0;
        let mut value = 0;

        while !self.exit_signal.load(Ordering::Relaxed) {
            let mut writer = self.list.writer().unwrap();
            writer.push_back(value);
            writer.push_front(value);

            node_count += 2;
            total_sum += 2 * value;
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
        let mut context = RcuContextMemb::new().unwrap();

        let mut node_count = 0;
        let mut total_sum = 0;

        loop {
            let mut writer = self.list.writer().unwrap();
            let value = writer.pop_back();
            let value = rcu_take_ownership!(&mut context, value);

            if let Some(value) = value {
                node_count += 1;
                total_sum += *value;
            } else if self.publisher_count.load(Ordering::Relaxed) == 0 {
                break;
            }
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
        exit.store(true, Ordering::Relaxed);
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
