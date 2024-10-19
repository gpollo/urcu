mod bindings {
    #![allow(warnings)]

    use libc::{pthread_attr_t, pthread_mutex_t};

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod wfcq {
    pub use crate::bindings::{
        __cds_wfcq_head as __Head,
        cds_wfcq_head as Head,
        cds_wfcq_head_ptr_t as HeadPtr,
        cds_wfcq_node as Node,
        cds_wfcq_ret as Ret,
        cds_wfcq_state as State,
        cds_wfcq_tail as Tail,
    };

    pub use crate::bindings::{
        cds_wfcq_ret_CDS_WFCQ_RET_DEST_EMPTY as RET_DEST_EMPTY,
        cds_wfcq_ret_CDS_WFCQ_RET_DEST_NON_EMPTY as RET_DEST_NON_EMPTY,
        cds_wfcq_ret_CDS_WFCQ_RET_SRC_EMPTY as RET_SRC_EMPTY,
        cds_wfcq_ret_CDS_WFCQ_RET_WOULDBLOCK as RET_WOULDBLOCK,
        cds_wfcq_state_CDS_WFCQ_STATE_LAST as STATE_LAST,
    };

    pub use crate::bindings::{
        __cds_wfcq_dequeue_blocking as __dequeue_blocking,
        __cds_wfcq_dequeue_nonblocking as __dequeue_nonblocking,
        __cds_wfcq_dequeue_with_state_blocking as __dequeue_with_state_blocking,
        __cds_wfcq_dequeue_with_state_nonblocking as __dequeue_with_state_nonblocking,
        __cds_wfcq_first_blocking as __first_blocking,
        __cds_wfcq_first_nonblocking as __first_nonblocking,
        __cds_wfcq_init as __init,
        __cds_wfcq_next_blocking as __next_blocking,
        __cds_wfcq_next_nonblocking as __next_nonblocking,
        __cds_wfcq_splice_blocking as __splice_blocking,
        __cds_wfcq_splice_nonblocking as __splice_nonblocking,
        cds_wfcq_dequeue_blocking as dequeue_blocking,
        cds_wfcq_dequeue_lock as dequeue_lock,
        cds_wfcq_dequeue_unlock as dequeue_unlock,
        cds_wfcq_dequeue_with_state_blocking as dequeue_with_state_blocking,
        cds_wfcq_destroy as destroy,
        cds_wfcq_empty as empty,
        cds_wfcq_enqueue as enqueue,
        cds_wfcq_init as init,
        cds_wfcq_node_init as node_init,
        cds_wfcq_splice_blocking as splice_blocking,
    };
}

pub mod wfq {
    pub use crate::bindings::{cds_wfq_node as Node, cds_wfq_queue as Queue};

    pub use crate::bindings::{
        __cds_wfq_dequeue_blocking as __dequeue_blocking,
        cds_wfq_dequeue_blocking as dequeue_blocking,
        cds_wfq_destroy as destroy,
        cds_wfq_enqueue as enqueue,
        cds_wfq_init as init,
        cds_wfq_node_init as node_init,
    };
}

pub mod wfs {
    pub use crate::bindings::{
        __cds_wfs_stack as __Stack,
        cds_wfs_head as Head,
        cds_wfs_node as Node,
        cds_wfs_stack as Stack,
        cds_wfs_stack_ptr_t as StackPtr,
        cds_wfs_state as State,
    };

    pub use crate::bindings::cds_wfs_state_CDS_WFS_STATE_LAST as STATE_LAST;

    pub use crate::bindings::{
        __cds_wfs_init as __init,
        __cds_wfs_pop_all as __pop_all,
        __cds_wfs_pop_blocking as __pop_blocking,
        __cds_wfs_pop_nonblocking as __pop_nonblocking,
        __cds_wfs_pop_with_state_blocking as __pop_with_state_blocking,
        __cds_wfs_pop_with_state_nonblocking as __pop_with_state_nonblocking,
        cds_wfs_destroy as destroy,
        cds_wfs_empty as empty,
        cds_wfs_first as first,
        cds_wfs_init as init,
        cds_wfs_next_blocking as next_blocking,
        cds_wfs_next_nonblocking as next_nonblocking,
        cds_wfs_node_init as node_init,
        cds_wfs_pop_all_blocking as pop_all_blocking,
        cds_wfs_pop_blocking as pop_blocking,
        cds_wfs_pop_lock as pop_lock,
        cds_wfs_pop_unlock as pop_unlock,
        cds_wfs_pop_with_state_blocking as pop_with_state_blocking,
        cds_wfs_push as push,
    };
}

#[test]
fn symbols() {
    macro_rules! print_symbol {
        ($sym:expr) => {
            println!("{:?}: {}", $sym as *const (), stringify!($sym))
        };
    }

    print_symbol!(wfcq::__dequeue_nonblocking);
    print_symbol!(wfcq::__dequeue_blocking);
    print_symbol!(wfcq::__dequeue_with_state_blocking);
    print_symbol!(wfcq::__dequeue_with_state_nonblocking);
    print_symbol!(wfcq::__first_blocking);
    print_symbol!(wfcq::__first_nonblocking);
    print_symbol!(wfcq::__init);
    print_symbol!(wfcq::__next_blocking);
    print_symbol!(wfcq::__next_nonblocking);
    print_symbol!(wfcq::__splice_blocking);
    print_symbol!(wfcq::__splice_nonblocking);
    print_symbol!(wfcq::dequeue_blocking);
    print_symbol!(wfcq::dequeue_lock);
    print_symbol!(wfcq::dequeue_unlock);
    print_symbol!(wfcq::dequeue_with_state_blocking);
    print_symbol!(wfcq::destroy);
    print_symbol!(wfcq::empty);
    print_symbol!(wfcq::enqueue);
    print_symbol!(wfcq::init);
    print_symbol!(wfcq::node_init);
    print_symbol!(wfcq::splice_blocking);

    print_symbol!(wfq::__dequeue_blocking);
    print_symbol!(wfq::dequeue_blocking);
    print_symbol!(wfq::destroy);
    print_symbol!(wfq::enqueue);
    print_symbol!(wfq::init);
    print_symbol!(wfq::node_init);

    print_symbol!(wfs::__init);
    print_symbol!(wfs::__pop_all);
    print_symbol!(wfs::__pop_blocking);
    print_symbol!(wfs::__pop_nonblocking);
    print_symbol!(wfs::__pop_with_state_blocking);
    print_symbol!(wfs::__pop_with_state_nonblocking);
    print_symbol!(wfs::destroy);
    print_symbol!(wfs::empty);
    print_symbol!(wfs::first);
    print_symbol!(wfs::init);
    print_symbol!(wfs::next_blocking);
    print_symbol!(wfs::next_nonblocking);
    print_symbol!(wfs::node_init);
    print_symbol!(wfs::pop_all_blocking);
    print_symbol!(wfs::pop_blocking);
    print_symbol!(wfs::pop_lock);
    print_symbol!(wfs::pop_unlock);
    print_symbol!(wfs::pop_with_state_blocking);
    print_symbol!(wfs::push);
}
