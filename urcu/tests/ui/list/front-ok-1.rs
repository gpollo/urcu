use urcu::flavor::RcuContextMemb;
use urcu::{RcuContext, RcuReadContext};
use urcu::RcuList;

fn main() {
    let context = RcuContextMemb::rcu_register().unwrap();
    
    let list = RcuList::<u32, RcuContextMemb>::new();
    let guard = context.rcu_read_lock();
    let front = list.front(&guard);
    log::info!("{:?}", front);
    drop(guard);
    drop(list);
}
