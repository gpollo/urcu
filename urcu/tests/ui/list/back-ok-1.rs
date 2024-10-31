use urcu::{RcuContext, RcuReadContext, DefaultContext};
use urcu::RcuList;

fn main() {
    let context = DefaultContext::rcu_register().unwrap();
    
    let list = RcuList::<u32>::new();
    let guard = context.rcu_read_lock();
    let back = list.back(&guard);
    log::info!("{:?}", back);
    drop(guard);
    drop(list);
}
