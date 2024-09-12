use urcu::flavor::RcuContextMemb;
use urcu::linked_list::RcuList;
use urcu::{rcu_take_ownership, RcuRef};

fn main() {
    let mut context = RcuContextMemb::new().unwrap();
    let list = RcuList::<u32, RcuContextMemb>::new();

    let mut writer = list.writer().unwrap();
    writer.push_front(10);
    writer.push_front(20);
    writer.push_front(30);
    writer.push_front(40);
    writer.push_front(50);
    writer.push_front(60);

    let v10 = writer.pop_back().unwrap();
    let v20 = writer.pop_back().unwrap();
    writer.pop_back().unwrap().defer_cleanup(&mut context);

    let (v10, v20) = rcu_take_ownership!(&mut context, v10, v20);
    assert_eq!(*v10, 10);
    assert_eq!(*v20, 20);
}
