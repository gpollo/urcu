use urcu::prelude::*;

fn main() {
    let context = RcuDefaultContext::rcu_register().unwrap();

    let list = RcuList::<u32>::new();
    let guard = context.rcu_read_lock();
    let front = list.front(&guard);
    log::info!("{:?}", front);
    drop(guard);
    drop(list);
}
