use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};

use crate::list::iterator::Iter;
use crate::list::raw::{RawIter, RawList, RawNode};
use crate::list::reference::Ref;
use crate::rcu::{DefaultContext, RcuContext, RcuReadContext};
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
pub struct RcuList<T, C = DefaultContext> {
    raw: RawList<T>,
    mutex: Mutex<()>,
    _unsend: PhantomUnsend<C>,
    _unsync: PhantomUnsync<C>,
}

impl<T, C> RcuList<T, C> {
    /// Creates a new RCU linked list.
    pub fn new() -> Arc<Self>
    where
        C: RcuContext,
    {
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
    pub fn contains(&self, x: &T, guard: &C::Guard<'_>) -> bool
    where
        T: PartialEq,
        C: RcuReadContext,
    {
        self.iter_forward(guard).any(|item| item == x)
    }

    fn with_mutex<F, R>(&self, callback: F) -> Result<R>
    where
        F: FnOnce() -> R,
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
    pub fn pop_back(&self) -> Result<Option<Ref<T, C>>>
    where
        T: Send,
        C: RcuContext,
    {
        self.with_mutex(|| {
            // SAFETY: There is mutual exclusion between writers.
            // SAFETY: The RCU grace period is enforced using `Ref<T, C>`.
            let node = unsafe { self.raw.remove_back() };

            NonNull::new(node).map(Ref::new)
        })
    }

    /// Removes an element from the fron of a list.
    ///
    /// #### Note
    ///
    /// This operation may block.
    pub fn pop_front(&self) -> Result<Option<Ref<T, C>>>
    where
        T: Send,
        C: RcuContext,
    {
        self.with_mutex(|| {
            // SAFETY: There is mutual exclusion between writers.
            // SAFETY: The RCU grace period is enforced using `Ref<T, C>`.
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
    pub fn back<'me, 'ctx, 'guard>(&'me self, guard: &'guard C::Guard<'ctx>) -> Option<&'guard T>
    where
        'me: 'guard,
        C: RcuReadContext,
    {
        let _ = guard;

        // SAFETY: The RCU critical section is enforced.
        // SAFETY: The node pointer can be converted to a reference.
        unsafe { self.raw.get_back().as_ref() }.map(|r| r.deref())
    }

    /// Provides a reference to the front element, or `None` if the list is empty.
    pub fn front<'me, 'ctx, 'guard>(&'me self, guard: &'guard C::Guard<'ctx>) -> Option<&'guard T>
    where
        'me: 'guard,
        C: RcuReadContext,
    {
        let _ = guard;

        // SAFETY: The RCU critical section is enforced.
        // SAFETY: The node pointer can be converted to a reference.
        unsafe { self.raw.get_front().as_ref() }.map(|r| r.deref())
    }

    /// Returns an iterator over the list.
    ///
    /// The iterator yields all items from back to front.
    pub fn iter_forward<'me, 'ctx, 'guard>(
        &'me self,
        guard: &'guard C::Guard<'ctx>,
    ) -> Iter<'ctx, 'guard, T, C, true>
    where
        'me: 'guard,
        C: RcuReadContext,
    {
        // SAFETY: The RCU critical section is enforced.
        Iter::new(unsafe { RawIter::<T, true>::from_back(&self.raw) }, guard)
    }

    /// Returns an iterator over the list.
    ///
    /// The iterator yields all items from front to back.
    pub fn iter_reverse<'me, 'ctx, 'guard>(
        &'me self,
        guard: &'guard C::Guard<'ctx>,
    ) -> Iter<'ctx, 'guard, T, C, false>
    where
        'me: 'guard,
        C: RcuReadContext,
    {
        // SAFETY: The RCU critical section is enforced.
        Iter::new(unsafe { RawIter::<T, false>::from_front(&self.raw) }, guard)
    }
}

/// #### Safety
///
/// An [`RcuList`] can be used to send `T` to another thread.
unsafe impl<T, C> Send for RcuList<T, C>
where
    T: Send,
    C: RcuContext,
{
}

/// #### Safety
///
/// An [`RcuList`] can be used to share `T` between threads.
unsafe impl<T, C> Sync for RcuList<T, C>
where
    T: Sync,
    C: RcuContext,
{
}

impl<T, C> Drop for RcuList<T, C> {
    fn drop(&mut self) {
        // SAFETY: The RCU grace period is not needed because there are no other readers.
        while let Some(mut ptr) = NonNull::new(unsafe { self.raw.remove_back() }) {
            drop(unsafe { Box::from_raw(ptr.as_mut()) });
        }
    }
}
