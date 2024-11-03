use urcu::prelude::*;

fn main() {
    let context = RcuDefaultFlavor::rcu_context_builder().with_read_context().register_thread().unwrap();

    let list = RcuList::<u32>::new();
    let guard = context.rcu_read_lock();
    let back = list.back(&guard);
    log::info!("{:?}", back);
    drop(guard);
    drop(list);
}
