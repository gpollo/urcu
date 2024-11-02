use crate::collections::list::container::RcuList;
use crate::rcu::context::{RcuContext, RcuReadContext};
use crate::rcu::default::RcuDefaultContext;
use crate::rcu::reference::RcuRef;

#[test]
fn peek() {
    let context = RcuDefaultContext::rcu_register().unwrap();
    let list = RcuList::<u32>::new();
    let guard = context.rcu_read_lock();

    assert_eq!(list.back(&guard), None);
    assert_eq!(list.front(&guard), None);
    assert!(list.is_empty());

    list.push_back(10).unwrap();
    assert_eq!(list.back(&guard), Some(&10));
    assert_eq!(list.front(&guard), Some(&10));
    assert!(!list.is_empty());

    list.push_back(20).unwrap();
    assert_eq!(list.back(&guard), Some(&20));
    assert_eq!(list.front(&guard), Some(&10));
    assert!(!list.is_empty());

    list.push_front(30).unwrap();
    assert_eq!(list.back(&guard), Some(&20));
    assert_eq!(list.front(&guard), Some(&30));
    assert!(!list.is_empty());

    list.push_back(40).unwrap();
    assert_eq!(list.back(&guard), Some(&40));
    assert_eq!(list.front(&guard), Some(&30));
    assert!(!list.is_empty());

    list.push_front(50).unwrap();
    assert_eq!(list.back(&guard), Some(&40));
    assert_eq!(list.front(&guard), Some(&50));
    assert!(!list.is_empty());

    list.pop_back().unwrap().call_cleanup(&context);
    assert_eq!(list.back(&guard), Some(&20));
    assert_eq!(list.front(&guard), Some(&50));
    assert!(!list.is_empty());

    list.pop_front().unwrap().call_cleanup(&context);
    assert_eq!(list.back(&guard), Some(&20));
    assert_eq!(list.front(&guard), Some(&30));
    assert!(!list.is_empty());

    list.pop_front().unwrap().call_cleanup(&context);
    assert_eq!(list.back(&guard), Some(&20));
    assert_eq!(list.front(&guard), Some(&10));
    assert!(!list.is_empty());

    list.pop_back().unwrap().call_cleanup(&context);
    assert_eq!(list.back(&guard), Some(&10));
    assert_eq!(list.front(&guard), Some(&10));
    assert!(!list.is_empty());

    list.pop_back().unwrap().call_cleanup(&context);
    assert_eq!(list.back(&guard), None);
    assert_eq!(list.front(&guard), None);
    assert!(list.is_empty());
}

#[test]
fn iter() {
    let context = RcuDefaultContext::rcu_register().unwrap();
    let list = RcuList::<u32>::new();
    let guard = context.rcu_read_lock();

    assert_eq!(
        list.iter_forward(&guard).copied().collect::<Vec<_>>(),
        vec![]
    );

    assert_eq!(
        list.iter_reverse(&guard).copied().collect::<Vec<_>>(),
        vec![]
    );

    list.push_back(140).unwrap();
    list.push_back(128).unwrap();
    list.push_back(174).unwrap();
    list.push_front(184).unwrap();
    list.push_back(150).unwrap();
    list.push_back(147).unwrap();
    list.push_front(105).unwrap();
    list.push_front(160).unwrap();
    list.push_back(120).unwrap();
    list.push_back(183).unwrap();

    assert_eq!(
        list.iter_forward(&guard).copied().collect::<Vec<_>>(),
        vec![183, 120, 147, 150, 174, 128, 140, 184, 105, 160]
    );

    assert_eq!(
        list.iter_reverse(&guard).copied().collect::<Vec<_>>(),
        vec![160, 105, 184, 140, 128, 174, 150, 147, 120, 183]
    );

    list.pop_back().unwrap().call_cleanup(&context);
    list.pop_back().unwrap().call_cleanup(&context);
    list.pop_back().unwrap().call_cleanup(&context);
    list.pop_front().unwrap().call_cleanup(&context);
    list.pop_back().unwrap().call_cleanup(&context);

    assert_eq!(
        list.iter_forward(&guard).copied().collect::<Vec<_>>(),
        vec![174, 128, 140, 184, 105]
    );

    assert_eq!(
        list.iter_reverse(&guard).copied().collect::<Vec<_>>(),
        vec![105, 184, 140, 128, 174]
    );

    list.pop_front().unwrap().call_cleanup(&context);
    list.pop_back().unwrap().call_cleanup(&context);
    list.pop_back().unwrap().call_cleanup(&context);
    list.pop_front().unwrap().call_cleanup(&context);
    list.push_back(142).unwrap();

    assert_eq!(
        list.iter_forward(&guard).copied().collect::<Vec<_>>(),
        vec![142, 140]
    );

    assert_eq!(
        list.iter_reverse(&guard).copied().collect::<Vec<_>>(),
        vec![140, 142]
    );

    list.pop_front().unwrap().call_cleanup(&context);
    list.pop_front().unwrap().call_cleanup(&context);

    assert_eq!(
        list.iter_forward(&guard).copied().collect::<Vec<_>>(),
        vec![]
    );

    assert_eq!(
        list.iter_reverse(&guard).copied().collect::<Vec<_>>(),
        vec![]
    );
}
