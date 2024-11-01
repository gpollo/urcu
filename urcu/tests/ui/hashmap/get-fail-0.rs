use urcu::{DefaultContext, RcuContext, RcuReadContext, RcuHashMap};

fn main() {
    let context = DefaultContext::rcu_register().unwrap();

    let map = RcuHashMap::<u32, u32>::new().unwrap();
    let guard = context.rcu_read_lock();
    let value = map.get(&0, &guard);
    drop(map);
    log::info!("{:?}", value);
    drop(guard);
}
