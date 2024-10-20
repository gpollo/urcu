use std::ffi::{c_int, c_void};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::ptr::NonNull;

use anyhow::{bail, Result};
use container_of::container_of;
use urcu_cds_sys::lfht;

use crate::rcu::api::RcuUnsafe;
use crate::rcu::RcuContext;
use crate::utility::{PhantomUnsend, PhantomUnsync};

//////////////////////
// helper functions //
//////////////////////

fn hash_of<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

unsafe extern "C" fn key_eq<K, V>(handle_ptr: *mut lfht::Node, key_ptr: *const c_void) -> c_int
where
    K: Eq,
{
    // SAFETY: The pointer is never null.
    // SAFETY: The pointer is valid for the duration of the reference..
    let node = unsafe { RawNode::<K, V>::from_handle(handle_ptr).as_ref_unchecked() };

    // SAFETY: The pointer is never null.
    // SAFETY: The pointer is valid for the duration of the reference..
    let key = unsafe { (key_ptr as *const K).as_ref_unchecked() };

    if &node.key == key {
        1
    } else {
        0
    }
}

//////////////////
// raw wrappers //
//////////////////

pub struct RawNodeHandle {
    handle: *mut lfht::Node,
    key: *const c_void,
    key_hash: u64,
}

pub struct RawNode<K, V> {
    handle: lfht::Node,
    pub(crate) key: K,
    pub(crate) value: V,
}

impl<K, V> RawNode<K, V> {
    fn new(key: K, value: V) -> Box<Self> {
        let mut node = Box::new(Self {
            key,
            value,
            handle: lfht::Node::default(),
        });

        // SAFETY: The pointer is non-null.
        unsafe { lfht::node_init(&mut node.handle) };

        node
    }

    fn to_handle(self: Box<Self>) -> RawNodeHandle
    where
        K: Hash,
    {
        let node = Box::into_raw(self);
        let node = unsafe { node.as_mut_unchecked() };

        RawNodeHandle {
            handle: &mut node.handle,
            key: &node.key as *const K as *const c_void,
            key_hash: hash_of(&node.key),
        }
    }

    /// #### Safety
    ///
    /// The pointer must not be null.
    unsafe fn from_handle(handle: *mut lfht::Node) -> *mut Self {
        container_of!(handle, Self, handle)
    }

    pub fn as_refs(&self) -> (&K, &V) {
        (&self.key, &self.value)
    }
}

pub struct RawIter<'a, K, V, C> {
    handle: lfht::Iter,
    map: &'a RawMap<K, V, C>,
    _unsend: PhantomUnsend<(K, V, C)>,
    _unsync: PhantomUnsync<(K, V, C)>,
}

impl<'a, K, V, C> RawIter<'a, K, V, C> {
    fn new<F>(map: &'a RawMap<K, V, C>, init: F) -> Self
    where
        F: FnOnce(*mut lfht::Iter),
    {
        let mut iterator = Self {
            map,
            handle: Default::default(),
            _unsend: PhantomData,
            _unsync: PhantomData,
        };

        init(&mut iterator.handle);
        iterator
    }

    pub fn get(&mut self) -> *mut RawNode<K, V> {
        // SAFETY: The iterator pointer is non-null.
        let node = unsafe { lfht::iter_get_node(&mut self.handle) };

        if node.is_null() {
            std::ptr::null_mut()
        } else {
            // SAFETY: The node pointer is non-null.
            unsafe { RawNode::<K, V>::from_handle(node) }
        }
    }

    pub fn next(&mut self) {
        // SAFETY: The hashmap pointer is non-null.
        // SAFETY: The iterator pointer is non-null.
        unsafe { lfht::next(self.map.handle, &mut self.handle) }
    }

    pub fn del(&mut self) -> *mut RawNode<K, V> {
        // SAFETY: The iterator pointer is non-null.
        let node = unsafe { lfht::iter_get_node(&mut self.handle) };

        // SAFETY: The iterator pointer is non-null.
        // SAFETY: The node pointer is non-null.
        unsafe {
            if lfht::del(self.map.handle, node) < 0 {
                std::ptr::null_mut()
            } else {
                RawNode::from_handle(node)
            }
        }
    }
}

pub struct RawMap<K, V, C> {
    handle: *mut lfht::Handle,
    _unsend: PhantomUnsend<(K, V, C)>,
    _unsync: PhantomUnsync<(K, V, C)>,
}

impl<K, V, C> RawMap<K, V, C> {
    const INIT_FLAGS: i32 = (lfht::ACCOUNTING | lfht::AUTO_RESIZE) as i32;
    const INIT_SIZE: u64 = 1;
    const MIN_NR_ALLOC_BUCKETS: u64 = 1;
    const MAX_NR_BUCKETS: u64 = 0;

    pub fn new() -> Result<Self>
    where
        C: RcuContext,
    {
        let handle = unsafe {
            lfht::new_flavor(
                Self::INIT_SIZE,
                Self::MIN_NR_ALLOC_BUCKETS,
                Self::MAX_NR_BUCKETS,
                Self::INIT_FLAGS,
                C::Unsafe::unchecked_rcu_api(),
                std::ptr::null_mut(),
            )
        };

        if handle.is_null() {
            bail!("failed to allocate RCU hash table");
        }

        Ok(Self {
            handle,
            _unsend: PhantomData,
            _unsync: PhantomData,
        })
    }

    /// #### Safety
    ///
    /// The caller must be in a RCU read-side critical section.
    ///
    /// The caller must wait for a RCU grace period before taking ownership of the old value.
    pub unsafe fn add_replace(&self, key: K, value: V) -> *mut RawNode<K, V>
    where
        K: Eq + Hash,
    {
        let node = RawNode::new(key, value).to_handle();

        // SAFETY: All pointers are non-null.
        let node = unsafe {
            lfht::add_replace(
                self.handle,
                node.key_hash,
                Some(key_eq::<K, V>),
                node.key,
                node.handle,
            )
        };

        if node.is_null() {
            std::ptr::null_mut()
        } else {
            // SAFETY: The node pointer is non-null.
            RawNode::from_handle(node)
        }
    }

    /// #### Safety
    ///
    /// The caller must be in a RCU read-side critical section.
    pub unsafe fn lookup(&self, key: &K) -> RawIter<K, V, C>
    where
        K: Eq + Hash,
    {
        RawIter::new(self, |iter| {
            // SAFETY: All pointers are non-null.
            unsafe {
                lfht::lookup(
                    self.handle,
                    hash_of(key),
                    Some(key_eq::<K, V>),
                    key as *const K as *const c_void,
                    iter,
                );
            }
        })
    }

    /// #### Safety
    ///
    /// The caller must be in a RCU read-side critical section.
    pub unsafe fn iter(&self) -> RawIter<K, V, C> {
        RawIter::new(self, |iter| {
            // SAFETY: All pointers are non-null.
            unsafe { lfht::first(self.handle, iter) }
        })
    }

    /// #### Safety
    ///
    /// The caller must be in a RCU read-side critical section.
    ///
    /// The caller must wait for a RCU grace period before taking ownership of the old value.
    pub unsafe fn del(&self, mut node: NonNull<RawNode<K, V>>) -> *mut RawNode<K, V>
    where
        C: RcuContext,
    {
        // SAFETY: The iterator pointer is non-null.
        // SAFETY: The node pointer is non-null.
        unsafe {
            if lfht::del(self.handle, &mut node.as_mut().handle) < 0 {
                std::ptr::null_mut()
            } else {
                node.as_mut()
            }
        }
    }

    /// #### Safety
    ///
    /// The caller must be in a RCU read-side critical section.
    ///
    /// The caller must wait for a RCU grace period before taking ownership of the old values.
    pub unsafe fn del_all(&self) -> Vec<NonNull<RawNode<K, V>>> {
        let mut iter = self.iter();
        let mut refs = Vec::new();

        loop {
            if iter.get().is_null() {
                break;
            }

            NonNull::new(iter.del())
                .iter()
                .copied()
                .for_each(|node| refs.push(node));
            iter.next();
        }

        refs
    }

    pub fn clone(&mut self) -> Self {
        Self {
            handle: self.handle,
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
    }

    /// #### Safety
    ///
    /// The caller must be a read-registered RCU thread.
    ///
    /// The caller must not be in a RCU critical section.
    pub unsafe fn destroy(&mut self) {
        unsafe { lfht::destroy(self.handle, std::ptr::null_mut()) };
    }
}

/// #### Safety
///
/// It is safe to send the wrapper to another thread if the key/value are [`Send`].
unsafe impl<K, V, C> Send for RawMap<K, V, C>
where
    K: Send,
    V: Send,
{
}

/// #### Safety
///
/// It is safe to send the wrapper to another thread if the key/value are [`Sync`].
unsafe impl<K, V, C> Sync for RawMap<K, V, C>
where
    K: Sync,
    V: Sync,
{
}
