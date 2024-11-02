use urcu::prelude::*;

fn main() {
    let context = DefaultContext::rcu_register().unwrap();

    let list = RcuList::<u32>::new();
    let guard = context.rcu_read_lock();
    let back = list.back(&guard);
    log::info!("{:?}", back);
    drop(list);
    drop(guard);
}
