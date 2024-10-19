use urcu::{DefaultContext, RcuContext, RcuHashMap};

fn main() {
    let context = DefaultContext::rcu_register().unwrap();

    let map = RcuHashMap::<u32, u32>::new().unwrap();
    let guard = context.rcu_read_lock();
    let mut iter = map.iter(&guard);
    log::info!("{:?}", iter.next());
    drop(map);
    drop(guard);
}