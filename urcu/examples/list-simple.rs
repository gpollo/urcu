use urcu::flavor::RcuContextMemb;
use urcu::RcuList;
use urcu::{RcuContext, RcuRef};

fn main() {
    let mut context = RcuContextMemb::rcu_register().unwrap();
    let list = RcuList::<u32, RcuContextMemb>::new();

    list.push_front(10).unwrap();
    list.push_front(20).unwrap();
    list.push_front(30).unwrap();
    list.push_front(40).unwrap();
    list.push_front(50).unwrap();
    list.push_front(60).unwrap();

    let v10 = list.pop_back().unwrap().unwrap();
    let v20 = list.pop_back().unwrap().unwrap();
    list.pop_back().unwrap().defer_cleanup(&mut context);
    list.pop_back().unwrap().call_cleanup(&context);
    list.pop_back().unwrap().safe_cleanup();

    let (v10, v20) = (v10, v20).take_ownership(&mut context);
    assert_eq!(*v10, 10);
    assert_eq!(*v20, 20);
}
