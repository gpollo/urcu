use urcu::flavor::RcuContextMemb;
use urcu::RcuContext;
use urcu::RcuList;

fn main() {
    let context = RcuContextMemb::rcu_register().unwrap();
    
    let list = RcuList::<u32, RcuContextMemb>::new();
    let guard = context.rcu_read_lock();
    let mut iter = list.iter_forward(&guard);
    println!("{:?}", iter.next());
    drop(list);
    drop(guard);
}
