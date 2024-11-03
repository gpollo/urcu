use crate::collections::queue::container::RcuQueue;
use crate::rcu::context::RcuReadContext;
use crate::rcu::default::RcuDefaultFlavor;
use crate::rcu::flavor::RcuFlavor;

#[test]
fn simple() {
    let context = RcuDefaultFlavor::rcu_context_builder()
        .with_read_context()
        .register_thread()
        .unwrap();

    let queue = RcuQueue::<u32>::new();
    let guard = context.rcu_read_lock();

    assert_eq!(queue.pop(&guard).as_deref(), None);
    assert_eq!(queue.pop(&guard).as_deref(), None);

    queue.push(10, &guard);

    assert_eq!(queue.pop(&guard).as_deref(), Some(&10));
    assert_eq!(queue.pop(&guard).as_deref(), None);
    assert_eq!(queue.pop(&guard).as_deref(), None);

    queue.push(20, &guard);
    queue.push(30, &guard);

    assert_eq!(queue.pop(&guard).as_deref(), Some(&20));

    queue.push(40, &guard);
    queue.push(50, &guard);

    assert_eq!(queue.pop(&guard).as_deref(), Some(&30));
    assert_eq!(queue.pop(&guard).as_deref(), Some(&40));
    assert_eq!(queue.pop(&guard).as_deref(), Some(&50));
    assert_eq!(queue.pop(&guard).as_deref(), None);
    assert_eq!(queue.pop(&guard).as_deref(), None);
}
