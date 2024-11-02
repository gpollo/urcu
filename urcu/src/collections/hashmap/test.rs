use crate::collections::hashmap::container::RcuHashMap;
use crate::rcu::context::{RcuContext, RcuReadContext};
use crate::rcu::default::RcuDefaultContext;
use crate::rcu::reference::RcuRef;

macro_rules! assert_sorted_eq {
    ($left:expr, $right:expr) => {
        let mut left = $left;
        left.sort();

        let mut right = $right;
        right.sort();

        assert_eq!(left, right);
    };
}

#[test]
fn get() {
    let context = RcuDefaultContext::rcu_register().unwrap();
    let hashmap = RcuHashMap::<u32, u32>::new().unwrap();
    let guard = context.rcu_read_lock();

    assert_eq!(hashmap.get(&2367, &guard), None);
    assert_eq!(hashmap.get(&6068, &guard), None);
    assert_eq!(hashmap.get(&9823, &guard), None);
    assert_eq!(hashmap.get(&7038, &guard), None);
    assert_eq!(hashmap.get(&7321, &guard), None);
    assert_eq!(hashmap.get(&9810, &guard), None);

    hashmap.insert(2367, 9848, &guard).call_cleanup(&context);
    hashmap.insert(6068, 4733, &guard).call_cleanup(&context);
    hashmap.insert(9823, 4944, &guard).call_cleanup(&context);
    assert_eq!(hashmap.get(&2367, &guard), Some(&9848));
    assert_eq!(hashmap.get(&6068, &guard), Some(&4733));
    assert_eq!(hashmap.get(&9823, &guard), Some(&4944));
    assert_eq!(hashmap.get(&7038, &guard), None);
    assert_eq!(hashmap.get(&7321, &guard), None);
    assert_eq!(hashmap.get(&9810, &guard), None);

    hashmap.insert(7038, 6341, &guard).call_cleanup(&context);
    hashmap.insert(7321, 2556, &guard).call_cleanup(&context);
    hashmap.remove(&9823, &guard).call_cleanup(&context);
    assert_eq!(hashmap.get(&2367, &guard), Some(&9848));
    assert_eq!(hashmap.get(&6068, &guard), Some(&4733));
    assert_eq!(hashmap.get(&9823, &guard), None);
    assert_eq!(hashmap.get(&7038, &guard), Some(&6341));
    assert_eq!(hashmap.get(&7321, &guard), Some(&2556));
    assert_eq!(hashmap.get(&9810, &guard), None);

    hashmap.remove(&2367, &guard).call_cleanup(&context);
    hashmap.remove(&7038, &guard).call_cleanup(&context);
    hashmap.remove(&7321, &guard).call_cleanup(&context);
    hashmap.remove(&9823, &guard).call_cleanup(&context);
    hashmap.insert(9810, 7691, &guard).call_cleanup(&context);
    assert_eq!(hashmap.get(&2367, &guard), None);
    assert_eq!(hashmap.get(&6068, &guard), Some(&4733));
    assert_eq!(hashmap.get(&9823, &guard), None);
    assert_eq!(hashmap.get(&7038, &guard), None);
    assert_eq!(hashmap.get(&7321, &guard), None);
    assert_eq!(hashmap.get(&9810, &guard), Some(&7691));

    hashmap.remove(&7038, &guard).call_cleanup(&context);
    hashmap.remove(&6068, &guard).call_cleanup(&context);
    hashmap.remove(&9810, &guard).call_cleanup(&context);
    hashmap.remove(&9823, &guard).call_cleanup(&context);
    assert_eq!(hashmap.get(&2367, &guard), None);
    assert_eq!(hashmap.get(&6068, &guard), None);
    assert_eq!(hashmap.get(&9823, &guard), None);
    assert_eq!(hashmap.get(&7038, &guard), None);
    assert_eq!(hashmap.get(&7321, &guard), None);
    assert_eq!(hashmap.get(&9810, &guard), None);
}

#[test]
fn contains() {
    let context = RcuDefaultContext::rcu_register().unwrap();
    let hashmap = RcuHashMap::<u32, u32>::new().unwrap();
    let guard = context.rcu_read_lock();

    assert!(!hashmap.contains(&6847, &guard));
    assert!(!hashmap.contains(&6614, &guard));
    assert!(!hashmap.contains(&1330, &guard));
    assert!(!hashmap.contains(&5154, &guard));
    assert!(!hashmap.contains(&8996, &guard));

    hashmap.insert(6847, 6228, &guard).call_cleanup(&context);
    assert!(hashmap.contains(&6847, &guard));
    assert!(!hashmap.contains(&6614, &guard));
    assert!(!hashmap.contains(&1330, &guard));
    assert!(!hashmap.contains(&5154, &guard));
    assert!(!hashmap.contains(&8996, &guard));

    hashmap.insert(6614, 1920, &guard).call_cleanup(&context);
    hashmap.insert(1330, 2524, &guard).call_cleanup(&context);
    assert!(hashmap.contains(&6847, &guard));
    assert!(hashmap.contains(&6614, &guard));
    assert!(hashmap.contains(&1330, &guard));
    assert!(!hashmap.contains(&5154, &guard));
    assert!(!hashmap.contains(&8996, &guard));

    hashmap.insert(5154, 7117, &guard).call_cleanup(&context);
    hashmap.insert(8996, 5158, &guard).call_cleanup(&context);
    hashmap.remove(&6614, &guard).call_cleanup(&context);
    hashmap.remove(&6614, &guard).call_cleanup(&context);
    assert!(hashmap.contains(&6847, &guard));
    assert!(!hashmap.contains(&6614, &guard));
    assert!(hashmap.contains(&1330, &guard));
    assert!(hashmap.contains(&5154, &guard));
    assert!(hashmap.contains(&8996, &guard));

    hashmap.remove(&6847, &guard).call_cleanup(&context);
    hashmap.remove(&1330, &guard).call_cleanup(&context);
    hashmap.remove(&5154, &guard).call_cleanup(&context);
    hashmap.remove(&5154, &guard).call_cleanup(&context);
    hashmap.remove(&6847, &guard).call_cleanup(&context);
    hashmap.remove(&8996, &guard).call_cleanup(&context);
    assert!(!hashmap.contains(&6847, &guard));
    assert!(!hashmap.contains(&6614, &guard));
    assert!(!hashmap.contains(&1330, &guard));
    assert!(!hashmap.contains(&5154, &guard));
    assert!(!hashmap.contains(&8996, &guard));
}

#[test]
fn iter() {
    let context = RcuDefaultContext::rcu_register().unwrap();
    let hashmap = RcuHashMap::<u32, u32>::new().unwrap();
    let guard = context.rcu_read_lock();

    assert_sorted_eq!(hashmap.iter(&guard).collect::<Vec<_>>(), vec![]);

    hashmap.insert(5837, 4209, &guard).call_cleanup(&context);
    assert_sorted_eq!(
        hashmap.iter(&guard).collect::<Vec<_>>(),
        vec![(&5837, &4209)]
    );

    hashmap.insert(6030, 9028, &guard).call_cleanup(&context);
    hashmap.insert(8423, 3333, &guard).call_cleanup(&context);
    assert_sorted_eq!(
        hashmap.iter(&guard).collect::<Vec<_>>(),
        vec![(&5837, &4209), (&6030, &9028), (&8423, &3333)]
    );

    hashmap.remove(&99, &guard).call_cleanup(&context);
    assert_sorted_eq!(
        hashmap.iter(&guard).collect::<Vec<_>>(),
        vec![(&5837, &4209), (&6030, &9028), (&8423, &3333)]
    );

    hashmap.remove(&6030, &guard).call_cleanup(&context);
    assert_sorted_eq!(
        hashmap.iter(&guard).collect::<Vec<_>>(),
        vec![(&5837, &4209), (&8423, &3333)]
    );

    hashmap.insert(8423, 3333, &guard).call_cleanup(&context);
    hashmap.insert(8423, 3333, &guard).call_cleanup(&context);
    assert_sorted_eq!(
        hashmap.iter(&guard).collect::<Vec<_>>(),
        vec![(&5837, &4209), (&8423, &3333)]
    );

    hashmap.remove(&5837, &guard).call_cleanup(&context);
    hashmap.remove(&8423, &guard).call_cleanup(&context);
    hashmap.remove(&8423, &guard).call_cleanup(&context);
    hashmap.remove(&8423, &guard).call_cleanup(&context);
    hashmap.remove(&5837, &guard).call_cleanup(&context);
    assert_sorted_eq!(hashmap.iter(&guard).collect::<Vec<_>>(), vec![]);
}
