use crate::queue::container::RcuQueue;
use crate::rcu::{DefaultContext, RcuContext, RcuReadContext};

#[test]
fn simple() {
    let context = DefaultContext::rcu_register().unwrap();
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
