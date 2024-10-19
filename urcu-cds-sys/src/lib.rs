mod bindings {
    #![allow(warnings)]

    use libc::{pthread_attr_t, pthread_mutex_t};
    use urcu_sys::{RcuFlavorApi as rcu_flavor_struct, RcuHead as rcu_head};

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod hlist {
    pub use crate::bindings::{cds_hlist_head as Head, cds_hlist_node as Node};

    pub use crate::bindings::{
        cds_hlist_add_head as add_head,
        cds_hlist_add_head_rcu as add_head_rcu,
        cds_hlist_del as del,
        cds_hlist_del_rcu as del_rcu,
        CDS_INIT_HLIST_HEAD as init_head,
    };
}

pub mod lfht {
    pub use crate::bindings::{
        cds_lfht as Handle,
        cds_lfht_iter as Iter,
        cds_lfht_mm_type as MemoryType,
        cds_lfht_node as Node,
    };

    pub use crate::bindings::cds_lfht_match_fct as MatchFn;

    pub use crate::bindings::{
        _cds_lfht_new as _new,
        cds_lfht_add as add,
        cds_lfht_add_replace as add_replace,
        cds_lfht_add_unique as add_unique,
        cds_lfht_count_nodes as count_nodes,
        cds_lfht_del as del,
        cds_lfht_destroy as destroy,
        cds_lfht_first as first,
        cds_lfht_is_node_deleted as is_node_deleted,
        cds_lfht_iter_get_node as iter_get_node,
        cds_lfht_lookup as lookup,
        cds_lfht_new_flavor as new_flavor,
        cds_lfht_next as next,
        cds_lfht_next_duplicate as next_duplicate,
        cds_lfht_node_init as node_init,
        cds_lfht_node_init_deleted as node_init_deleted,
        cds_lfht_replace as replace,
        cds_lfht_resize as resize,
    };

    pub use crate::bindings::{
        cds_lfht_mm_chunk as MM_CHUNK,
        cds_lfht_mm_mmap as MM_MMAP,
        cds_lfht_mm_order as MM_ORDER,
        CDS_LFHT_ACCOUNTING as ACCOUNTING,
        CDS_LFHT_AUTO_RESIZE as AUTO_RESIZE,
    };
}

pub mod lfq {
    pub use crate::bindings::{cds_lfq_node_rcu as NodeRcu, cds_lfq_queue_rcu as QueueRcu};

    pub use crate::bindings::{
        cds_lfq_dequeue_rcu as dequeue_rcu,
        cds_lfq_destroy_rcu as destroy_rcu,
        cds_lfq_enqueue_rcu as enqueue_rcu,
        cds_lfq_init_rcu as init_rcu,
        cds_lfq_node_init_rcu as node_init_rcu,
    };
}

pub mod lfs {
    pub use crate::bindings::{
        __cds_lfs_stack as __Stack,
        cds_lfs_head as Head,
        cds_lfs_node as Node,
        cds_lfs_node_rcu as NodeRcu,
        cds_lfs_stack as Stack,
        cds_lfs_stack_ptr_t as StackPtr,
        cds_lfs_stack_rcu as StackRcu,
    };

    pub use crate::bindings::{
        __cds_lfs_init as __init,
        __cds_lfs_pop as __pop,
        __cds_lfs_pop_all as __pop_all,
        cds_lfs_destroy as destroy,
        cds_lfs_empty as empty,
        cds_lfs_init as init,
        cds_lfs_init_rcu as init_rcu,
        cds_lfs_node_init as node_init,
        cds_lfs_node_init_rcu as node_init_rcu,
        cds_lfs_pop_all_blocking as pop_all_blocking,
        cds_lfs_pop_blocking as pop_blocking,
        cds_lfs_pop_lock as pop_lock,
        cds_lfs_pop_rcu as pop_rcu,
        cds_lfs_pop_unlock as pop_unlock,
        cds_lfs_push as push,
        cds_lfs_push_rcu as push_rcu,
    };
}

pub mod list {
    pub use crate::bindings::cds_list_head as Head;

    pub use crate::bindings::{
        __cds_list_del as __del,
        cds_list_add as add,
        cds_list_add_rcu as add_rcu,
        cds_list_add_tail as add_tail,
        cds_list_add_tail_rcu as add_tail_rcu,
        cds_list_del as del,
        cds_list_del_init as del_init,
        cds_list_del_rcu as del_rcu,
        cds_list_empty as empty,
        cds_list_move as r#move,
        cds_list_replace as replace,
        cds_list_replace_init as replace_init,
        cds_list_replace_rcu as replace_rcu,
        cds_list_splice as splice,
    };
}

#[test]
fn symbols() {
    macro_rules! print_symbol {
        ($sym:expr) => {
            println!("{:?}: {}", $sym as *const (), stringify!($sym))
        };
    }

    print_symbol!(hlist::add_head);
    print_symbol!(hlist::add_head_rcu);
    print_symbol!(hlist::del);
    print_symbol!(hlist::del_rcu);
    print_symbol!(hlist::init_head);

    print_symbol!(lfht::_new);
    print_symbol!(lfht::add);
    print_symbol!(lfht::add_replace);
    print_symbol!(lfht::add_unique);
    print_symbol!(lfht::count_nodes);
    print_symbol!(lfht::del);
    print_symbol!(lfht::destroy);
    print_symbol!(lfht::first);
    print_symbol!(lfht::is_node_deleted);
    print_symbol!(lfht::iter_get_node);
    print_symbol!(lfht::lookup);
    print_symbol!(lfht::new_flavor);
    print_symbol!(lfht::next);
    print_symbol!(lfht::next_duplicate);
    print_symbol!(lfht::node_init);
    print_symbol!(lfht::node_init_deleted);
    print_symbol!(lfht::replace);
    print_symbol!(lfht::resize);

    print_symbol!(lfq::dequeue_rcu);
    print_symbol!(lfq::destroy_rcu);
    print_symbol!(lfq::enqueue_rcu);
    print_symbol!(lfq::init_rcu);
    print_symbol!(lfq::node_init_rcu);

    print_symbol!(lfs::__init);
    print_symbol!(lfs::__pop);
    print_symbol!(lfs::__pop_all);
    print_symbol!(lfs::destroy);
    print_symbol!(lfs::empty);
    print_symbol!(lfs::init);
    print_symbol!(lfs::init_rcu);
    print_symbol!(lfs::node_init);
    print_symbol!(lfs::node_init_rcu);
    print_symbol!(lfs::pop_all_blocking);
    print_symbol!(lfs::pop_blocking);
    print_symbol!(lfs::pop_lock);
    print_symbol!(lfs::pop_rcu);
    print_symbol!(lfs::pop_unlock);
    print_symbol!(lfs::push);
    print_symbol!(lfs::push_rcu);

    print_symbol!(list::__del);
    print_symbol!(list::add);
    print_symbol!(list::add_rcu);
    print_symbol!(list::add_tail);
    print_symbol!(list::add_tail_rcu);
    print_symbol!(list::del);
    print_symbol!(list::del_init);
    print_symbol!(list::del_rcu);
    print_symbol!(list::empty);
    print_symbol!(list::r#move);
    print_symbol!(list::replace);
    print_symbol!(list::replace_init);
    print_symbol!(list::replace_rcu);
    print_symbol!(list::splice);
}
