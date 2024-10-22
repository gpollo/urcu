use urcu::{DefaultContext, RcuContext, RcuReadContext, RcuStack};

fn main() {
    let context = DefaultContext::rcu_register().unwrap();

    let stack = RcuStack::<u32>::new();
    let guard = context.rcu_read_lock();
    let peek = stack.peek(&guard);
    log::info!("{:?}", peek);
    drop(stack);
    drop(guard);
}