use std::marker::PhantomData;
use std::ops::Deref;
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
    pub fn new(data: T) -> Arc<Self> {
        Arc::new(Self {
            ptr: AtomicPtr::new(Box::into_raw(Box::new(data))),
            _unsend: PhantomData,
            _unsync: PhantomData,
        })
    }

    pub fn accessor<'a>(&'a self, guard: &'a C::Guard<'a>) -> Accessor<'a, T, C>
    where
        C: RcuContext,
    {
        Accessor {
            boxed: self,
            _guard: guard,
            _unsend: PhantomData,
            _unsync: PhantomData,
        }
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

/// A RCU read protected accessor to a [`RcuBox`].
pub struct Accessor<'a, T, C>
where
    C: RcuContext,
{
    boxed: &'a RcuBox<T, C>,
    _guard: &'a C::Guard<'a>,
    _unsend: PhantomUnsend<C>,
    _unsync: PhantomUnsync<C>,
}

impl<'a, T, C> Accessor<'a, T, C>
where
    C: RcuContext,
{
    pub fn replace(&self, data: T) -> Ref<T, C>
    where
        T: Send,
    {
        let new_ptr = Box::into_raw(Box::new(data));
        let old_ptr = self.boxed.ptr.swap(new_ptr, Ordering::Release);
        Ref::new(old_ptr)
    }
}

impl<'a, T, C> Deref for Accessor<'a, T, C>
where
    C: RcuContext,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The atomic pointer is never null except during dropping.
        unsafe { &*self.boxed.ptr.load(Ordering::Acquire) }
    }
}
