use urcu::prelude::*;

fn main() {
    let context = DefaultContext::rcu_register().unwrap();

    let stack = RcuStack::<u32>::new();
    let guard = context.rcu_read_lock();
    let peek = stack.peek(&guard);
    drop(guard);
    log::info!("{:?}", peek);
    drop(stack);
}