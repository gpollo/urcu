use std::ffi::{c_int, c_void};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use anyhow::{bail, Result};
use container_of::container_of;
use urcu_sys::lfht;
use urcu_sys::lfht::{HashTable, HashTableIterator, HashTableNode};

use crate::RcuContext;

//////////////////////
// helper functions //
//////////////////////

fn hash_of<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

unsafe extern "C" fn key_eq<K, V>(handle_ptr: *mut HashTableNode, key_ptr: *const c_void) -> c_int
where
    K: Eq,
{
    let node = RawNode::<K, V>::from_handle_unchecked(handle_ptr);
    let key = unsafe { &*(key_ptr as *const K) };

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
    handle_ptr: *mut HashTableNode,
    key_ptr: *const c_void,
    key_hash: u64,
}

pub struct RawNode<K, V> {
    handle: HashTableNode,
    key: K,
    value: V,
}

impl<K, V> RawNode<K, V> {
    fn new(key: K, value: V) -> Box<Self> {
        let mut node = Box::new(Self {
            key,
            value,
            handle: HashTableNode::default(),
        });

        unsafe {
            lfht::node_init(&mut node.handle);
        }

        node
    }

    fn to_handle(self: Box<Self>) -> RawNodeHandle
    where
        K: Hash,
    {
        let node_ptr = Box::into_raw(self);
        let node = unsafe { &mut *node_ptr };
        let handle_ptr = &mut node.handle;
        let key_ptr = &node.key as *const K as *const c_void;
        let key_hash = hash_of(&node.key);

        RawNodeHandle {
            handle_ptr,
            key_ptr,
            key_hash,
        }
    }

    fn from_handle_unchecked<'a>(handle_ptr: *const HashTableNode) -> &'a Self {
        unsafe { &*container_of!(handle_ptr, Self, handle) }
    }

    fn from_handle<'a>(handle_ptr: *const HashTableNode) -> Option<&'a Self> {
        if !handle_ptr.is_null() {
            Some(Self::from_handle_unchecked(handle_ptr))
        } else {
            None
        }
    }

    fn from_handle_mut_unchecked<'a>(handle_ptr: *mut HashTableNode) -> &'a mut Self {
        unsafe { &mut *container_of!(handle_ptr, Self, handle) }
    }

    fn from_handle_mut<'a>(handle_ptr: *mut HashTableNode) -> Option<&'a mut Self> {
        if !handle_ptr.is_null() {
            Some(Self::from_handle_mut_unchecked(handle_ptr))
        } else {
            None
        }
    }

    pub fn as_refs(&self) -> (&K, &V) {
        (&self.key, &self.value)
    }
}

impl<K, V> Deref for RawNode<K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<K, V> DerefMut for RawNode<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub struct RawIterator<'a, K, V, C> {
    handle: HashTableIterator,
    map: &'a RawMap<K, V, C>,
    // Also prevents auto-trait implementation of [`Send`] and [`Sync`].
    _key: PhantomData<*const K>,
    _value: PhantomData<*const V>,
    _context: PhantomData<*const C>,
}

impl<'a, K, V, C> RawIterator<'a, K, V, C> {
    fn new<F>(map: &'a RawMap<K, V, C>, init: F) -> Self
    where
        F: FnOnce(*mut HashTableIterator),
    {
        let mut iterator = Self {
            map,
            handle: Default::default(),
            _key: PhantomData,
            _value: PhantomData,
            _context: PhantomData,
        };

        init(&mut iterator.handle);
        iterator
    }

    pub fn get(&mut self) -> Option<&'a RawNode<K, V>> {
        RawNode::<K, V>::from_handle(unsafe { lfht::iter_get_node(&mut self.handle) })
    }

    pub fn get_mut(&mut self) -> Option<&'a mut RawNode<K, V>> {
        RawNode::<K, V>::from_handle_mut(unsafe { lfht::iter_get_node(&mut self.handle) })
    }

    pub fn next(&mut self) {
        unsafe { lfht::next(self.map.handle, &mut self.handle) }
    }
}

pub struct RawMap<K, V, C> {
    handle: *mut HashTable,
    // Also prevents auto-trait implementation of [`Send`] and [`Sync`].
    _key: PhantomData<*const K>,
    _value: PhantomData<*const V>,
    _context: PhantomData<*const C>,
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
                C::rcu_api(),
                std::ptr::null_mut(),
            )
        };

        if handle.is_null() {
            bail!("failed to allocate RCU hash table");
        }

        Ok(Self {
            handle,
            _key: PhantomData,
            _value: PhantomData,
            _context: PhantomData,
        })
    }

    /// #### Safety
    ///
    /// The caller must be in an RCU read-side critical section.
    ///
    /// The caller must wait for an RCU grace period before taking ownership of the old value.
    pub unsafe fn add_replace(&self, key: K, value: V) -> Option<*mut RawNode<K, V>>
    where
        K: Eq + Hash,
    {
        let node = RawNode::new(key, value).to_handle();

        RawNode::from_handle_mut(unsafe {
            lfht::add_replace(
                self.handle,
                node.key_hash,
                Some(key_eq::<K, V>),
                node.key_ptr,
                node.handle_ptr,
            )
        })
        .map(|node| node as *mut RawNode<K, V>)
    }

    /// #### Safety
    ///
    /// The caller must be in an RCU read-side critical section.
    pub unsafe fn lookup(&self, key: &K) -> RawIterator<K, V, C>
    where
        K: Eq + Hash,
    {
        RawIterator::new(self, |iter| unsafe {
            lfht::lookup(
                self.handle,
                hash_of(key),
                Some(key_eq::<K, V>),
                key as *const K as *const c_void,
                iter,
            );
        })
    }

    /// #### Safety
    ///
    /// The caller must be in an RCU read-side critical section.
    pub unsafe fn iter(&self) -> RawIterator<K, V, C> {
        RawIterator::new(self, |iter| unsafe { lfht::first(self.handle, iter) })
    }

    /// #### Safety
    ///
    /// The caller must be in an RCU read-side critical section.
    ///
    /// The caller must wait for an RCU grace period before taking ownership of the old value.
    pub unsafe fn del(&self, node_ptr: *mut RawNode<K, V>) -> Option<*mut RawNode<K, V>>
    where
        C: RcuContext,
    {
        unsafe {
            let node = &mut *node_ptr;
            if lfht::del(self.handle, &mut node.handle) < 0 {
                None
            } else {
                Some(node_ptr)
            }
        }
    }
}
