use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};

use crate::collections::list::iterator::Iter;
use crate::collections::list::raw::{RawIter, RawList, RawNode};
use crate::collections::list::reference::Ref;
use crate::rcu::default::DefaultFlavor;
use crate::rcu::flavor::RcuFlavor;
use crate::rcu::guard::RcuGuard;
use crate::utility::*;

/// Defines a RCU doubly linked list.
///
/// This linked list supports multiple concurrents readers at any time, but only a single
/// writer at a time. The list uses an internal lock for writing operations.
///
/// # Limitations
///
/// ##### Mutable References
///
/// Because there might always be readers borrowing a node's data, it is impossible
/// to get a mutable references to the data inside the linked list. You should design
/// the type stored in the list with [interior mutabillity] that can be shared between
/// threads.
///
/// [interior mutabillity]: https://doc.rust-lang.org/reference/interior-mutability.html
///
/// ##### List Length
///
/// Because a writer might concurrently modify the list, the amount of node might change
/// at any moment. To prevent user error (e.g. allocate an array for each node), there is
/// no `.len()` method.
///
/// # Safety
///
/// It is safe to send an `Arc<RcuList<T>>` to a non-registered RCU thread. A non-registered
/// thread may drop an `RcuList<T>` without calling any RCU primitives since lifetime rules
/// prevent any other thread from accessing a RCU reference.
pub struct RcuList<T, F = DefaultFlavor> {
    raw: RawList<T>,
    mutex: Mutex<()>,
    _unsend: PhantomUnsend<F>,
    _unsync: PhantomUnsync<F>,
}

impl<T, F> RcuList<T, F>
where
    F: RcuFlavor,
{
    /// Creates a new RCU linked list.
    pub fn new() -> Arc<Self> {
        let mut list = Arc::new(RcuList {
            // SAFETY: Initialisation is properly called.
            raw: unsafe { RawList::new() },
            mutex: Default::default(),
            _unsend: PhantomData,
            _unsync: PhantomData,
        });

        // SAFETY: Initialisation occurs when raw list is in a stable memory location.
        // SAFETY: All the nodes are removed upon dropping.
        unsafe { Arc::<Self>::get_mut(&mut list).unwrap().raw.init() };

        list
    }

    /// Returns `true` if the list contains an element equal to the given value.
    pub fn contains<G>(&self, x: &T, guard: &G) -> bool
    where
        T: PartialEq,
        G: RcuGuard<Flavor = F>,
    {
        self.iter_forward(guard).any(|item| item == x)
    }

    fn with_mutex<C, R>(&self, callback: C) -> Result<R>
    where
        C: FnOnce() -> R,
    {
        match self.mutex.lock() {
            Err(_) => bail!("mutex of the list has been poisoned"),
            Ok(guard) => {
                let result = callback();
                drop(guard);
                Ok(result)
            }
        }
    }

    /// Adds an element to the back of a list.
    ///
    /// #### Note
    ///
    /// This operation may block.
    pub fn push_back(&self, data: T) -> Result<()> {
        self.with_mutex(|| {
            // SAFETY: There is mutual exclusion between writers.
            unsafe {
                let node = RawNode::new(data);
                self.raw.insert_back(node);
            }
        })
    }

    /// Adds an element to the front of a list.
    ///
    /// #### Note
    ///
    /// This operation may block.
    pub fn push_front(&self, data: T) -> Result<()> {
        self.with_mutex(|| {
            // SAFETY: There is mutual exclusion between writers.
            unsafe {
                let node = RawNode::new(data);
                self.raw.insert_front(node);
            }
        })
    }

    /// Removes an element from the back of a list.
    ///
    /// #### Note
    ///
    /// This operation may block.
    pub fn pop_back(&self) -> Result<Option<Ref<T, F>>>
    where
        T: Send,
    {
        self.with_mutex(|| {
            // SAFETY: There is mutual exclusion between writers.
            // SAFETY: The RCU grace period is enforced using `Ref<T, F>`.
            let node = unsafe { self.raw.remove_back() };

            NonNull::new(node).map(Ref::new)
        })
    }

    /// Removes an element from the fron of a list.
    ///
    /// #### Note
    ///
    /// This operation may block.
    pub fn pop_front(&self) -> Result<Option<Ref<T, F>>>
    where
        T: Send,
    {
        self.with_mutex(|| {
            // SAFETY: There is mutual exclusion between writers.
            // SAFETY: The RCU grace period is enforced using `Ref<T, F>`.
            let node = unsafe { self.raw.remove_front() };

            NonNull::new(node).map(Ref::new)
        })
    }

    /// Returns `true` if the list is empty.
    ///
    /// #### Note
    ///
    /// * This operation computes linearly in *O*(*1*) time.
    pub fn is_empty(&self) -> bool {
        self.raw.empty()
    }

    /// Provides a reference to the back element, or `None` if the list is empty.
    pub fn back<'me, 'guard, G>(&'me self, guard: &'guard G) -> Option<&'guard T>
    where
        'me: 'guard,
        G: RcuGuard<Flavor = F>,
    {
        let _ = guard;

        // SAFETY: The RCU critical section is enforced.
        // SAFETY: The node pointer can be converted to a reference.
        unsafe { self.raw.get_back().as_ref() }.map(|r| r.deref())
    }

    /// Provides a reference to the front element, or `None` if the list is empty.
    pub fn front<'me, 'guard, G>(&'me self, guard: &'guard G) -> Option<&'guard T>
    where
        'me: 'guard,
        G: RcuGuard<Flavor = F>,
    {
        let _ = guard;

        // SAFETY: The RCU critical section is enforced.
        // SAFETY: The node pointer can be converted to a reference.
        unsafe { self.raw.get_front().as_ref() }.map(|r| r.deref())
    }

    /// Returns an iterator over the list.
    ///
    /// The iterator yields all items from back to front.
    pub fn iter_forward<'me, 'guard, G>(&'me self, guard: &'guard G) -> Iter<'guard, T, G, true>
    where
        'me: 'guard,
        G: RcuGuard<Flavor = F>,
    {
        // SAFETY: The RCU critical section is enforced.
        Iter::new(unsafe { RawIter::<T, true>::from_back(&self.raw) }, guard)
    }

    /// Returns an iterator over the list.
    ///
    /// The iterator yields all items from front to back.
    pub fn iter_reverse<'me, 'guard, G>(&'me self, guard: &'guard G) -> Iter<'guard, T, G, false>
    where
        'me: 'guard,
        G: RcuGuard,
    {
        // SAFETY: The RCU critical section is enforced.
        Iter::new(unsafe { RawIter::<T, false>::from_front(&self.raw) }, guard)
    }
}

/// #### Safety
///
/// An [`RcuList`] can be used to send `T` to another thread.
unsafe impl<T, F> Send for RcuList<T, F>
where
    T: Send,
    F: RcuFlavor,
{
}

/// #### Safety
///
/// An [`RcuList`] can be used to share `T` between threads.
unsafe impl<T, F> Sync for RcuList<T, F>
where
    T: Sync,
    F: RcuFlavor,
{
}

impl<T, F> Drop for RcuList<T, F> {
    fn drop(&mut self) {
        // SAFETY: The RCU grace period is not needed because there are no other readers.
        while let Some(mut ptr) = NonNull::new(unsafe { self.raw.remove_back() }) {
            drop(unsafe { Box::from_raw(ptr.as_mut()) });
        }
    }
}
