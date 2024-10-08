use std::marker::PhantomData;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;

use crate::boxed::reference::Ref;
use crate::rcu::RcuContext;
use crate::utility::{PhantomUnsend, PhantomUnsync};

/// Defines a RCU-enabled [`Box`].
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
/// # Safety
///
/// It is safe to send an `Arc<RcuBox<T>>` to a non-registered RCU thread. A non-registered
/// thread may drop an `RcuBox<T>` without calling any RCU primitives since lifetime rules
/// prevent any other thread from accessing an RCU reference.
pub struct RcuBox<T, C> {
    ptr: AtomicPtr<T>,
    _unsend: PhantomUnsend<C>,
    _unsync: PhantomUnsync<C>,
}

impl<T, C> RcuBox<T, C> {
    /// Creates a new RCU box.
    pub fn new(data: T) -> Arc<Self> {
        Arc::new(Self {
            ptr: AtomicPtr::new(Box::into_raw(Box::new(data))),
            _unsend: PhantomData,
            _unsync: PhantomData,
        })
    }

    /// Returns a immutable reference to the data.
    pub fn as_ref<'a>(&'a self, guard: &'a C::Guard<'a>) -> &'a T
    where
        C: RcuContext,
    {
        let _ = guard;

        // SAFETY: The underlying pointer is never null.
        unsafe { self.ptr.load(Ordering::Acquire).as_ref_unchecked() }
    }

    /// Replaces the underlying data atomically.
    pub fn replace(&self, data: T) -> Ref<T, C>
    where
        T: Send,
        C: RcuContext,
    {
        let new_ptr = Box::into_raw(Box::new(data));
        let old_ptr = self.ptr.swap(new_ptr, Ordering::Release);
        Ref::new(old_ptr)
    }
}

/// #### Safety
///
/// An [`RcuBox`] can be used to send `T` to another thread.
unsafe impl<T, C> Send for RcuBox<T, C> where T: Send {}

/// #### Safety
///
/// An [`RcuBox`] can be used to share `T` between threads.
unsafe impl<T, C> Sync for RcuBox<T, C> where T: Sync {}

impl<T, C> Drop for RcuBox<T, C> {
    fn drop(&mut self) {
        // SAFETY: The underlying pointer is never null.
        unsafe {
            let _ = Box::from_raw(self.ptr.load(Ordering::Relaxed));
        }
    }
}
