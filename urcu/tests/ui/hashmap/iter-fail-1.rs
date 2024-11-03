use urcu::prelude::*;

fn main() {
    let context = RcuDefaultFlavor::rcu_context_builder().with_read_context().register_thread().unwrap();

    let map = RcuHashMap::<u32, u32>::new().unwrap();
    let guard = context.rcu_read_lock();
    let mut iter = map.iter(&guard);
    drop(guard);
    log::info!("{:?}", iter.next());
    drop(map);
}
