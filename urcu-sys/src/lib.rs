mod bindings {
    #![allow(warnings)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use bindings::{
    rcu_flavor_struct as RcuFlavor,
    rcu_head as RcuHead,
    urcu_atfork as RcuAtFork,
    urcu_gp_poll_state as RcuPollState,
};

pub mod lfht {
    pub use crate::bindings::{
        cds_lfht as HashTable,
        cds_lfht_iter as HashTableIterator,
        cds_lfht_mm_type as HashTableMemoryModel,
        cds_lfht_node as HashTableNode,
    };

    pub use crate::bindings::{
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
        cds_lfht_new as new,
        cds_lfht_new_flavor as new_flavor,
        cds_lfht_next as next,
        cds_lfht_next_duplicate as next_duplicate,
        cds_lfht_node_init as node_init,
        cds_lfht_node_init_deleted as node_init_deleted,
        cds_lfht_replace as replace,
        cds_lfht_resize as resize,
    };

    pub use crate::bindings::{
        cds_lfht_mm_chunk as MEMORY_MODEL_CHUNK,
        cds_lfht_mm_mmap as MEMORY_MODEL_MMAP,
        cds_lfht_mm_order as MEMORY_MODLE_ORDER,
        CDS_LFHT_ACCOUNTING as ACCOUNTING,
        CDS_LFHT_AUTO_RESIZE as AUTO_RESIZE,
    };
}

/*************/
/* rculfhash */
/*************/

// #define cds_lfht_for_each(ht, iter, node)				\
// 	for (cds_lfht_first(ht, iter),					\
// 			node = cds_lfht_iter_get_node(iter);		\
// 		node != NULL;						\
// 		cds_lfht_next(ht, iter),				\
// 			node = cds_lfht_iter_get_node(iter))

// #define cds_lfht_for_each_duplicate(ht, hash, match, key, iter, node)	\
// 	for (cds_lfht_lookup(ht, hash, match, key, iter),		\
// 			node = cds_lfht_iter_get_node(iter);		\
// 		node != NULL;						\
// 		cds_lfht_next_duplicate(ht, match, key, iter),		\
// 			node = cds_lfht_iter_get_node(iter))

// #define cds_lfht_for_each_entry(ht, iter, pos, member)			\
// 	for (cds_lfht_first(ht, iter),					\
// 			pos = caa_container_of(cds_lfht_iter_get_node(iter), \
// 					__typeof__(*(pos)), member);	\
// 		cds_lfht_iter_get_node(iter) != NULL;			\
// 		cds_lfht_next(ht, iter),				\
// 			pos = caa_container_of(cds_lfht_iter_get_node(iter), \
// 					__typeof__(*(pos)), member))

// #define cds_lfht_for_each_entry_duplicate(ht, hash, match, key,		\
// 				iter, pos, member)			\
// 	for (cds_lfht_lookup(ht, hash, match, key, iter),		\
// 			pos = caa_container_of(cds_lfht_iter_get_node(iter), \
// 					__typeof__(*(pos)), member);	\
// 		cds_lfht_iter_get_node(iter) != NULL;			\
// 		cds_lfht_next_duplicate(ht, match, key, iter),		\
// 			pos = caa_container_of(cds_lfht_iter_get_node(iter), \
// 					__typeof__(*(pos)), member))
