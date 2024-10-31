use urcu::context::RcuContextMemb;
use urcu::{RcuContext, RcuReadContext};
use urcu::RcuList;

fn main() {
    let context = RcuContextMemb::rcu_register().unwrap();
    
    let list = RcuList::<u32, RcuContextMemb>::new();
    let guard = context.rcu_read_lock();
    let back = list.back(&guard);
    drop(guard);
    log::info!("{:?}", back);
    drop(list);
}
