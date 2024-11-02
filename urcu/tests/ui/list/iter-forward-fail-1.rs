use urcu::prelude::*;

fn main() {
    let context = RcuDefaultContext::rcu_register().unwrap();

    let list = RcuList::<u32>::new();
    let guard = context.rcu_read_lock();
    let mut iter = list.iter_forward(&guard);
    drop(guard);
    log::info!("{:?}", iter.next());
    drop(list);
}
