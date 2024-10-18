use urcu::{DefaultContext, RcuContext, RcuStack};

fn main() {
    let context = DefaultContext::rcu_register().unwrap();

    let stack = RcuStack::<u32>::new();
    let guard = context.rcu_read_lock();
    let peek = stack.peek(&guard);
    println!("{:?}", peek);
    drop(stack);
    drop(guard);
}