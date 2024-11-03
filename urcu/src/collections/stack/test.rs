use std::ops::Deref;

use crate::collections::stack::container::RcuStack;
use crate::rcu::context::RcuReadContext;
use crate::rcu::default::RcuDefaultFlavor;
use crate::rcu::flavor::RcuFlavor;
use crate::rcu::reference::RcuRef;

#[test]
fn peek() {
    let context = RcuDefaultFlavor::rcu_context_builder()
        .with_read_context()
        .register_thread()
        .unwrap();

    let stack = RcuStack::<u32>::new();
    let guard = context.rcu_read_lock();

    assert_eq!(stack.peek(&guard), None);
    assert!(stack.is_empty());

    stack.push(10);
    assert_eq!(stack.peek(&guard), Some(&10));
    assert!(!stack.is_empty());

    stack.push(20);
    assert_eq!(stack.peek(&guard), Some(&20));
    assert!(!stack.is_empty());

    stack.push(30);
    assert_eq!(stack.peek(&guard), Some(&30));
    assert!(!stack.is_empty());

    stack.pop(&guard).call_cleanup(&context);
    assert_eq!(stack.peek(&guard), Some(&20));
    assert!(!stack.is_empty());

    stack.pop(&guard).call_cleanup(&context);
    assert_eq!(stack.peek(&guard), Some(&10));
    assert!(!stack.is_empty());

    stack.push(40);
    assert_eq!(stack.peek(&guard), Some(&40));
    assert!(!stack.is_empty());

    stack.pop(&guard).call_cleanup(&context);
    assert_eq!(stack.peek(&guard), Some(&10));
    assert!(!stack.is_empty());

    stack.pop(&guard).call_cleanup(&context);
    assert_eq!(stack.peek(&guard), None);
    assert!(stack.is_empty());
}

#[test]
fn iter() {
    let context = RcuDefaultFlavor::rcu_context_builder()
        .with_read_context()
        .register_thread()
        .unwrap();

    let stack = RcuStack::<u32>::new();
    let guard = context.rcu_read_lock();

    assert_eq!(stack.iter(&guard).copied().collect::<Vec<_>>(), vec![]);

    stack.push(140);
    stack.push(128);
    stack.push(174);
    stack.push(184);
    stack.push(150);
    stack.push(147);
    stack.push(105);
    stack.push(160);
    stack.push(120);
    stack.push(183);

    assert_eq!(
        stack.iter(&guard).copied().collect::<Vec<_>>(),
        vec![183, 120, 160, 105, 147, 150, 184, 174, 128, 140]
    );

    stack.pop(&guard).unwrap().call_cleanup(&context);
    stack.pop(&guard).unwrap().call_cleanup(&context);
    stack.pop(&guard).unwrap().call_cleanup(&context);
    stack.pop(&guard).unwrap().call_cleanup(&context);
    stack.pop(&guard).unwrap().call_cleanup(&context);

    assert_eq!(
        stack.iter(&guard).copied().collect::<Vec<_>>(),
        vec![150, 184, 174, 128, 140]
    );

    stack.pop(&guard).unwrap().call_cleanup(&context);
    stack.pop(&guard).unwrap().call_cleanup(&context);
    stack.pop(&guard).unwrap().call_cleanup(&context);
    stack.push(142);

    assert_eq!(
        stack.iter(&guard).copied().collect::<Vec<_>>(),
        vec![142, 128, 140]
    );

    stack.pop(&guard).unwrap().call_cleanup(&context);
    stack.pop(&guard).unwrap().call_cleanup(&context);
    stack.pop(&guard).unwrap().call_cleanup(&context);

    assert_eq!(stack.iter(&guard).copied().collect::<Vec<_>>(), vec![]);
}

#[test]
fn iter_ref() {
    fn pop_all_nodes<C>(context: &mut C, stack: &RcuStack<u32>) -> Vec<u32>
    where
        C: RcuReadContext<Flavor = RcuDefaultFlavor>,
    {
        {
            let guard = context.rcu_read_lock();
            stack.pop_all(&guard).collect::<Vec<_>>()
        }
        .take_ownership(context)
        .iter()
        .map(|r| r.deref())
        .copied()
        .collect::<Vec<_>>()
    }

    let mut context = RcuDefaultFlavor::rcu_context_builder()
        .with_read_context()
        .register_thread()
        .unwrap();

    let stack = RcuStack::<u32>::new();

    assert_eq!(pop_all_nodes(&mut context, &stack), vec![]);

    stack.push(140);
    stack.push(128);
    stack.push(174);
    stack.push(184);
    stack.push(150);
    stack.push(147);
    stack.push(105);
    stack.push(160);
    stack.push(120);
    stack.push(183);

    assert_eq!(
        pop_all_nodes(&mut context, &stack),
        vec![183, 120, 160, 105, 147, 150, 184, 174, 128, 140]
    );
}
