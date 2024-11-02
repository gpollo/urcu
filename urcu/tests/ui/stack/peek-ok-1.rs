use urcu::prelude::*;

fn main() {
    let context = RcuDefaultContext::rcu_register().unwrap();

    let stack = RcuStack::<u32>::new();
    let guard = context.rcu_read_lock();
    let peek = stack.peek(&guard);
    log::info!("{:?}", peek);
    drop(guard);
    drop(stack);
}
