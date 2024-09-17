use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::NonNull;

use container_of::container_of;
use urcu_sys::RcuHead;

use crate::rcu::{RcuContext, RcuRef};

/// This trait defines a callback to be invoked after the next RCU grace period.
///
/// #### Implementation
///
/// RCU callbacks are put into a queue inside the RCU context. To do so, you need
/// to provide a pointer to an [`RcuHead`] that is owned by your type. Upon callback,
/// the same pointer will be provided to the callback. You can use [`container_of!`]
/// to get back the type implementing this trait.
///
/// #### Safety
///
/// When [`RcuCallback::configure`] is called, you must use [`Box::into_raw`] to
/// prevent the type to be freed. Upon execution of the callback, you must
/// use [`Box::from_raw`] to get back ownership and properly free up memory.
/// For example implementations, see [`RcuSimpleCallback`] and [`RcuCleanupCallback`].
pub unsafe trait RcuCallback {
    /// Configures the callback for execution.
    fn configure<F>(self: Box<Self>, func: F)
    where
        F: FnOnce(NonNull<RcuHead>, unsafe extern "C" fn(head: *mut RcuHead));
}

/// Defines a simple callback executed after the next RCU grace period.
pub struct RcuSimpleCallback<F> {
    func: Box<F>,
    head: RcuHead,
}

impl<F> RcuSimpleCallback<F> {
    /// Create a simple RCU callback.
    pub fn new(func: Box<F>) -> Box<Self> {
        Box::new(Self {
            func,
            head: Default::default(),
        })
    }

    unsafe extern "C" fn rcu_callback(head_ptr: *mut RcuHead)
    where
        F: FnOnce(),
    {
        // SAFETY: The pointers should always be valid.
        let node = Box::from_raw(container_of!(head_ptr, Self, head));

        (node.func)();
    }
}

/// #### Safety
///
/// The memory of [`Box<Self>`] is properly reclaimed upon the RCU callback.
unsafe impl<F> RcuCallback for RcuSimpleCallback<F>
where
    F: FnOnce(),
{
    fn configure<P>(self: Box<Self>, func: P)
    where
        P: FnOnce(NonNull<RcuHead>, unsafe extern "C" fn(head: *mut RcuHead)),
    {
        let node_ptr = Box::into_raw(self);
        let node = unsafe { &mut *node_ptr };

        unsafe {
            func(NonNull::new_unchecked(&mut node.head), Self::rcu_callback);
        }
    }
}

/// #### Safety
///
/// The callback can be sent to another thread if the reference implements [`Send`].
unsafe impl<F> Send for RcuSimpleCallback<F> where F: FnOnce() + Send {}

/// Defines a cleanup callback executed after the next RCU grace period.
///
/// Upon callback execution, it takes ownership of an [`RcuRef`] and drops the value.
pub struct RcuCleanupCallback<R, C> {
    data: R,
    head: RcuHead,
    _context: PhantomData<C>,
}

impl<R, C> RcuCleanupCallback<R, C>
where
    R: RcuRef<C>,
    C: RcuContext,
{
    /// Create a cleanup RCU callback.
    pub fn new(data: R) -> Box<Self> {
        Box::new(Self {
            data,
            head: Default::default(),
            _context: Default::default(),
        })
    }

    unsafe extern "C" fn rcu_callback(head_ptr: *mut RcuHead)
    where
        R: RcuRef<C>,
    {
        // SAFETY: The pointers should always be valid.
        let node = Box::from_raw(container_of!(head_ptr, Self, head));

        // SAFETY: This callback is always called after an RCU grace period.
        let data = node.data.take_ownership();

        drop(data);
    }
}

/// #### Safety
///
/// The memory of [`Box<Self>`] is properly reclaimed upon the RCU callback.
unsafe impl<R, C> RcuCallback for RcuCleanupCallback<R, C>
where
    R: RcuRef<C>,
    C: RcuContext,
{
    fn configure<F>(self: Box<Self>, func: F)
    where
        F: FnOnce(NonNull<RcuHead>, unsafe extern "C" fn(head: *mut RcuHead)),
    {
        let node_ptr = Box::into_raw(self);
        let node = unsafe { &mut *node_ptr };

        unsafe {
            func(NonNull::new_unchecked(&mut node.head), Self::rcu_callback);
        }
    }
}

/// #### Safety
///
/// The callback can be sent to another thread if the reference implements [`Send`].
unsafe impl<R: Send, C> Send for RcuCleanupCallback<R, C> {}

/// This trait defines a callback to be invoked after the next RCU grace period.
///
/// #### Implementation
///
/// Each RCU thread have an array of `(callback, data)` pointers. When the next RCU
/// grace period finishes, the thread goes over each of its entry and execute
/// `callback(data)`.
///
/// #### Safety
///
/// When [`RcuDeferConfig::configure`] is called, you must deliberately leak your type
/// (e.g. [`Box::into_raw`]) to prevent the memory from being freed. Upon execution
/// of the callback, you must get back ownership (e.g. [`Box::from_raw`]) and properly
/// free up memory. For an example, see [`RcuDeferSimple`].
pub unsafe trait RcuDeferConfig {
    /// Configures the callback for execution.
    fn configure<F>(self: Box<Self>, func: F)
    where
        F: FnOnce(NonNull<c_void>, unsafe extern "C" fn(head: *mut c_void));
}

/// Defines a simple defer callback executed after the next RCU grace period.
pub struct RcuDeferSimple<F, C> {
    func: F,
    // Also prevents Send+Sync auto-trait implementations.
    _context: PhantomData<*mut C>,
}

/// #### Safety
///
/// The memory of [`Box<Self>`] is properly reclaimed upon the RCU callback.
impl<F, C> RcuDeferSimple<F, C> {
    /// Creates a callback.
    pub fn new(func: F) -> Box<Self> {
        Box::new(Self {
            func,
            _context: PhantomData,
        })
    }

    unsafe extern "C" fn callback(ptr: *mut c_void)
    where
        F: FnOnce(),
    {
        // SAFETY: The pointers should always be valid.
        let node = Box::from_raw(ptr as *mut Self);

        (node.func)();
    }
}

unsafe impl<F, C> RcuDeferConfig for RcuDeferSimple<F, C>
where
    F: FnOnce(),
{
    fn configure<P>(self: Box<Self>, func: P)
    where
        P: FnOnce(NonNull<c_void>, unsafe extern "C" fn(head: *mut c_void)),
    {
        let ptr = Box::into_raw(self) as *mut c_void;

        // SAFETY: The pointer is never null.
        unsafe { func(NonNull::new_unchecked(ptr), Self::callback) }
    }
}
