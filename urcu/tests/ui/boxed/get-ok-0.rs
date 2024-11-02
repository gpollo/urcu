use urcu::prelude::*;

fn main() {
    let context = RcuDefaultContext::rcu_register().unwrap();

    let boxed = RcuBox::<u32>::new(0);
    let guard = context.rcu_read_lock();
    let value = boxed.get(&guard);
    log::info!("{:?}", value);
    drop(boxed);
    drop(guard);
}
