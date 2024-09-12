use std::any::TypeId;
use std::cell::Cell;
use std::marker::PhantomData;

use urcu_sys::RcuPollState;

use crate::rcu::callback::RcuCallback;
use crate::rcu::flavor::{DefaultFlavor, RcuFlavor};

#[allow(dead_code)]
struct RcuThreadFlavor<F: RcuFlavor>(PhantomData<F>);

impl<F: RcuFlavor> RcuThreadFlavor<F> {
    thread_local! {
        static STATUS: Cell<bool> = const { Cell::new(false) };
    }
}

/// Defines an handle to manually poll the RCU grace period.
pub struct RcuPoller<'a, T: 'a>
where
    T: RcuThread,
{
    state: RcuPollState,
    // Borrows `T` without actually borrowing it.
    _borrow: PhantomData<&'a T>,
    // Removes `Send` and `Sync` implementation.
    _pointer: PhantomData<*const T>,
}

impl<'a, T> RcuPoller<'a, T>
where
    T: RcuThread,
{
    /// #### Safety
    ///
    /// The caller must be an RCU read-registered thread.
    unsafe fn new(_thread: &'a T) -> Self {
        Self {
            state: T::Flavor::unchecked_rcu_poll_start(),
            _borrow: PhantomData,
            _pointer: PhantomData,
        }
    }
}

impl<'a, T> RcuPoller<'a, T>
where
    T: RcuThread,
{
    /// This trait is used to manually poll the RCU grace period.
    pub fn grace_period_finished(&self) -> bool {
        unsafe { T::Flavor::unchecked_rcu_poll_check(self.state) }
    }
}

/// Defines a guard for an RCU critical section.
pub struct RcuGuard<'a, C: 'a>
where
    C: RcuThread,
{
    // Borrows `T` without actually borrowing it.
    _borrow: PhantomData<&'a C>,
    // Removes `Send` and `Sync` implementation.
    _pointer: PhantomData<*const C>,
}

impl<'a, C: 'a> RcuGuard<'a, C>
where
    C: RcuThread,
{
    /// #### Safety
    ///
    /// The caller must be an RCU read-registered thread.
    unsafe fn new(_thread: &'a C) -> Self {
        // SAFETY: The RCU critical section is always unlocked upon dropping.
        unsafe {
            C::Flavor::unchecked_rcu_read_lock();
        }

        Self {
            _borrow: PhantomData,
            _pointer: PhantomData,
        }
    }
}

impl<'a, T: 'a> Drop for RcuGuard<'a, T>
where
    T: RcuThread,
{
    fn drop(&mut self) {
        // SAFETY: The RCU critical section is always locked upon creation.
        unsafe {
            T::Flavor::unchecked_rcu_read_unlock();
        }
    }
}

/// This trait defines a basic RCU context.
pub trait RcuThread: Sized {
    type Flavor: RcuFlavor;
}

/// This trait defines an RCU context for a writing thread.
pub trait RcuWriter: RcuThread {
    /// Waits until the RCU grace period is over.
    ///
    /// #### Note
    ///
    /// It cannot be called in an RCU critical section.
    fn rcu_synchronize(&mut self);
}

/// This trait defines an RCU context for a reading thread.
pub trait RcuReader: RcuWriter {
    /// Starts an RCU critical section.
    ///
    /// #### Note
    ///
    /// RCU critical sections may be nested.
    fn rcu_read_lock(&self) -> RcuGuard<'_, Self>;

    /// Creates an RCU grace period poller.
    ///
    /// #### Note
    ///
    /// It cannot be called in an RCU critical section.
    fn rcu_synchronize_poller(&self) -> RcuPoller<'_, Self>;

    /// Configures a callback to be called after the next RCU grace period is finished.
    ///
    /// #### Note
    ///
    /// The callback must be [`Send`] because it will be executed by a helper thread.
    fn rcu_call<Func>(callback: Box<Func>)
    where
        Func: RcuCallback + Send;
}

/// This trait defines an RCU context for a thread that can defer calls.
pub trait RcuDeferrer: RcuWriter {
    /// Configures a callback to be called after the next RCU grace period is finished.
    ///
    /// #### Note
    ///
    /// The function might internally execute an RCU syncronization and block.
    ///
    /// The callback is guaranteed to be executed on the current thread. (TODO: confirm)
    fn rcu_defer<Func>(&mut self, callback: Box<Func>)
    where
        Func: RcuCallback;
}

/// Defines a marker type for a context that supports RCU reading.
pub struct RcuFeatureRead;

/// Defines a marker type for a context that supports RCU deferring.
pub struct RcuFeatureDefer;

/// Defines a context for a specific thread.
/// 
/// ```
/// use urcu::{DefaultFlavor, RcuContext};
/// 
/// let mut context = RcuContext::<DefaultFlavor>::builder()
///     .with_read()
///     .with_defer()
///     .build()
///     .unwrap();
/// ```
pub struct RcuContext<F = DefaultFlavor, R = (), D = ()>(PhantomData<*mut (F, R, D)>)
where
    F: RcuFlavor,
    R: 'static,
    D: 'static;

impl<F, R, D> RcuThread for RcuContext<F, R, D>
where
    F: RcuFlavor,
{
    type Flavor = F;
}

impl<F, R, D> RcuWriter for RcuContext<F, R, D>
where
    Self: RcuThread,
    F: RcuFlavor,
    R: 'static,
    D: 'static,
{
    fn rcu_synchronize(&mut self) {
        // SAFETY: Rust borrowing rules prevent an RCU critical section from being active.
        unsafe {
            Self::Flavor::unchecked_rcu_synchronize();
        }
    }
}

impl<F, D> RcuReader for RcuContext<F, RcuFeatureRead, D>
where
    Self: RcuWriter,
    F: RcuFlavor,
    D: 'static,
{
    fn rcu_read_lock(&self) -> RcuGuard<'_, Self> {
        // SAFETY: The thread is registered upon creation.
        unsafe { RcuGuard::new(self) }
    }

    fn rcu_synchronize_poller(&self) -> RcuPoller<'_, Self> {
        // SAFETY: The caller is an RCU read-registered thread.
        unsafe { RcuPoller::new(self) }
    }

    fn rcu_call<Func>(callback: Box<Func>)
    where
        Func: RcuCallback + Send,
    {
        callback.configure(|mut head, func|
            // SAFETY: The thread is properly registered upon context's creation.
            // SAFETY: A barrier is called upon drop to execute remaining calls.
            unsafe {
                F::unchecked_rcu_call(Some(func), head.as_mut());
            });
    }
}

impl<F, R> RcuDeferrer for RcuContext<F, R, RcuFeatureDefer>
where
    Self: RcuWriter,
    F: RcuFlavor,
    R: 'static,
{
    fn rcu_defer<Func>(&mut self, callback: Box<Func>)
    where
        Func: RcuCallback,
    {
        // TODO: We could optimize deferred callbacks by not using [`RcuCallback`].
        type HeadPtrCallback = unsafe extern "C" fn(*mut urcu_sys::RcuHead);
        type VoidPtrCallback = unsafe extern "C" fn(*mut libc::c_void);
        callback.configure(|mut head, func|
            // SAFETY: Rust borrowing rules prevent an RCU critical section from being active.
            // SAFETY: A barrier is called upon drop to execute remaining calls.
            unsafe {
                F::unchecked_rcu_defer_call(
                    Some(std::mem::transmute::<HeadPtrCallback, VoidPtrCallback>(
                        func,
                    )),
                    head.as_mut() as *mut urcu_sys::RcuHead as *mut libc::c_void,
                );
            });
    }
}

impl<F, R, D> RcuContext<F, R, D>
where
    F: RcuFlavor,
    R: 'static,
    D: 'static,
{
    pub fn builder() -> RcuContextBuilder<F, R, D> {
        RcuContextBuilder(PhantomData)
    }

    fn new() -> Option<Self> {
        RcuThreadFlavor::<F>::STATUS.with(|status| {
            if status.get() {
                return None;
            }

            // SAFETY: This function is called once per thread.
            unsafe { F::unchecked_rcu_init() };

            if TypeId::of::<R>() == TypeId::of::<RcuFeatureRead>() {
                // SAFETY: The thread is unregistered upon dropping.
                unsafe { F::unchecked_rcu_read_register_thread() };
            }

            if TypeId::of::<D>() == TypeId::of::<RcuFeatureDefer>() {
                // SAFETY: The thread is unregistered upon dropping.
                unsafe { F::unchecked_rcu_defer_register_thread() };
            }

            status.set(true);
            Some(Self(PhantomData))
        })
    }
}

impl<F, R, D> Drop for RcuContext<F, R, D>
where
    F: RcuFlavor,
    R: 'static,
    D: 'static,
{
    fn drop(&mut self) {
        if TypeId::of::<D>() == TypeId::of::<RcuFeatureDefer>() {
            // SAFETY: The caller cannot be inside an RCU critical section when dropping.
            // SAFETY: The caller is an RCU defer-registered thread.
            unsafe { F::unchecked_rcu_defer_barrier() };

            // SAFETY: The caller is an RCU defer-registered thread.
            unsafe { F::unchecked_rcu_defer_unregister_thread() };
        }

        if TypeId::of::<R>() == TypeId::of::<RcuFeatureRead>() {
            // SAFETY: The caller is an RCU read-registered thread.
            // SAFETY: The caller cannot be in a callbacks since `Self` is not `Send`.
            unsafe { F::unchecked_rcu_call_barrier() };

            // SAFETY: The caller is an RCU read-registered thread.
            unsafe { F::unchecked_rcu_read_unregister_thread() };
        }
    }
}

/// Defines a builder pattern for configuring a context.
pub struct RcuContextBuilder<F, R = (), D = ()>(PhantomData<(F, R, D)>)
where
    F: RcuFlavor;

impl<F, R, D> RcuContextBuilder<F, R, D>
where
    F: RcuFlavor,
{
    pub fn build(self) -> Option<RcuContext<F, R, D>> {
        RcuContext::<F, R, D>::new()
    }
}

impl<F, D> RcuContextBuilder<F, (), D>
where
    F: RcuFlavor,
{
    pub fn with_read(self) -> RcuContextBuilder<F, RcuFeatureRead, D> {
        RcuContextBuilder(PhantomData)
    }
}

impl<F, R> RcuContextBuilder<F, R, ()>
where
    F: RcuFlavor,
{
    pub fn with_defer(self) -> RcuContextBuilder<F, R, RcuFeatureDefer> {
        RcuContextBuilder(PhantomData)
    }
}
