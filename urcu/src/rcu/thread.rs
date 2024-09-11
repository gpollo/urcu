use std::any::TypeId;
use std::cell::Cell;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::thread::LocalKey;

use urcu_sys::{RcuFlavorApi, RcuHead, RcuPollState};

use crate::rcu::callback::RcuCallback;

type RcuCallSignature<T> = Option<unsafe extern "C" fn(head: T)>;

pub unsafe trait RcuFlavor {
    /// Returns the status of the flavor for the **current thread**.
    ///
    /// #### Safety
    ///
    /// * If the status is `true`, the caller should not create a new context.
    /// * If the status is `false`, the caller may create a new context and register it.
    ///   * After the context is created, the caller must set the status to `true`.
    unsafe fn get_status<'a>() -> &'a LocalKey<Cell<bool>>;

    /// Registers a read-side RCU thread.
    ///
    /// #### Safety
    ///
    /// * The caller must unregister the thread manually.
    /// * The caller must not be a read-registered thread.
    unsafe fn unchecked_rcu_read_register_thread();

    /// Unregisters a read-side RCU thread.
    ///
    /// #### Safety
    ///
    /// * The caller must be a read-registered thread.
    unsafe fn unchecked_rcu_read_unregister_thread();

    /// Starts an RCU critical section.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU read-registered thread.
    /// * The caller must unlock the RCU critical section manually.
    unsafe fn unchecked_rcu_read_lock();

    /// Stops an RCU critical section.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU read-registered thread.
    /// * The caller must have activated an RCU critical section before.
    unsafe fn unchecked_rcu_read_unlock();

    /// Registers a defer-enabled RCU thread.
    ///
    /// #### Safety
    ///
    /// * The caller must unregister the thread manually.
    /// * The caller must not be an RCU defer-registered thread.
    unsafe fn unchecked_rcu_defer_register_thread();

    /// Unregisters a defer-enabled RCU thread.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU defer-registered thread.
    unsafe fn unchecked_rcu_defer_unregister_thread();

    /// Executes a call after the next RCU grace period.
    ///
    /// #### Note
    ///
    /// The callback will be executed on the same thread. If the internal queue is full
    /// the call might block and the callback will be executed immediatly. In such case,
    /// [`RcuFlavor::unchecked_rcu_synchronize`] will be called internally.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU defer-registered thread.
    /// * The caller must execute a defer barrier to prevent leaks.
    /// * The caller must not be inside an RCU critical section.
    unsafe fn unchecked_rcu_defer_call(func: RcuCallSignature<*mut c_void>, ptr: *mut c_void);

    /// Wait for all RCU deferred callbacks initiated by the current thread.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU defer-registered thread.
    /// * The caller must not be inside an RCU critical section.
    unsafe fn unchecked_rcu_defer_barrier();

    /// Waits until the RCU grace period is over.
    ///
    /// #### Safety
    ///
    /// * The caller must not be inside an RCU critical section.
    unsafe fn unchecked_rcu_synchronize();

    /// Creates an [`RcuPollState`] used for checking if the grace period has ended.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU read-registered thread.
    unsafe fn unchecked_rcu_poll_start() -> RcuPollState;

    /// Polls if the grace period has ended.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU read-registered thread.
    /// * The caller must use a [`RcuPollState`] of the same flavor.
    unsafe fn unchecked_rcu_poll_check(state: RcuPollState) -> bool;

    /// Executes a call after the next RCU grace period.
    ///
    /// #### Note
    ///
    /// This call nevers blocks because the callback will be executed on an helper thread.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU read-registered thread.
    /// * The caller must execute a call barrier to prevent leaks.
    unsafe fn unchecked_rcu_call(func: RcuCallSignature<*mut RcuHead>, ptr: *mut RcuHead);

    /// Wait for all RCU callbacks initiated before the call by any thread to be completed.
    ///
    /// #### Safety
    ///
    /// * The caller must be an RCU read-registered thread.
    /// * The caller must not be within a callback.
    unsafe fn unchecked_rcu_call_barrier();

    /// Returns the API list for this RCU flavor.
    fn rcu_api() -> &'static RcuFlavorApi;
}

pub struct RcuPoller<'a, T: 'a>(PhantomData<&'a T>, RcuPollState)
where
    T: RcuThreadFlavor;

impl<'a, T> RcuPoller<'a, T>
where
    T: RcuThreadFlavor,
{
    /// #### Safety
    ///
    /// The caller is responsible for ensuring it is called from a registered RCU read-side thread.
    unsafe fn new(_thread: &'a T) -> Self {
        Self(PhantomData, T::Flavor::unchecked_rcu_poll_start())
    }
}

impl<'a, T> RcuPoller<'a, T>
where
    T: RcuThreadFlavor,
{
    fn grace_period_finished(&self) -> bool {
        unsafe { T::Flavor::unchecked_rcu_poll_check(self.1) }
    }
}

pub struct RcuGuard<'a, T: 'a>(PhantomData<&'a T>)
where
    T: RcuThreadFlavor;

impl<'a, T: 'a> RcuGuard<'a, T>
where
    T: RcuThreadFlavor,
{
    /// #### Safety
    ///
    /// The caller is responsible for ensuring it is called from a registered RCU read-side thread.
    unsafe fn new(_thread: &'a T) -> Self {
        // SAFETY: The RCU critical section is unlocked upon dropping.
        T::Flavor::unchecked_rcu_read_lock();

        Self(PhantomData)
    }
}

impl<'a, T: 'a> Drop for RcuGuard<'a, T>
where
    T: RcuThreadFlavor,
{
    fn drop(&mut self) {
        // SAFETY: The RCU critical section is always taken upon creation.
        unsafe {
            T::Flavor::unchecked_rcu_read_unlock();
        }
    }
}

pub trait RcuThreadFlavor: Sized {
    type Flavor: RcuFlavor;
}

pub trait RcuWriter: RcuThreadFlavor {
    /// Waits until the RCU grace period is over.
    ///
    /// #### Note
    ///
    /// It cannot be called in an RCU critical section.
    fn rcu_synchronize(&mut self);
}

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

pub trait RcuDeferrer: RcuWriter {
    /// Configures a callback to be called after the next RCU grace period is finished.
    ///
    /// #### Note
    ///
    /// The function might internally call [`RcuContext::rcu_synchronize`] and block.
    ///
    /// The callback is guaranteed to be executed on the current thread.
    fn rcu_defer<Func>(&mut self, callback: Box<Func>)
    where
        Func: RcuCallback;
}

pub struct RcuFeatureRead;
pub struct RcuFeatureDefer;

struct RcuThread<F, R, D>(PhantomData<*mut (F, R, D)>)
where
    F: RcuFlavor,
    R: 'static,
    D: 'static;

impl<F, R, D> RcuThreadFlavor for RcuThread<F, R, D>
where
    F: RcuFlavor,
{
    type Flavor = F;
}

impl<F, R, D> RcuWriter for RcuThread<F, R, D>
where
    Self: RcuThreadFlavor,
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

impl<F, D> RcuReader for RcuThread<F, RcuFeatureRead, D>
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
        // SAFETY: The thread is registered upon creation.
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

impl<F, R> RcuDeferrer for RcuThread<F, R, RcuFeatureDefer>
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

impl<F, R, D> RcuThread<F, R, D>
where
    F: RcuFlavor,
    R: 'static,
    D: 'static,
{
    pub fn new() -> Option<Self> {
        // SAFETY: Only 1 context may be created per thread.
        let status = unsafe { F::get_status() };

        status.with(|status| {
            if status.get() {
                return None;
            }

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

impl<F, R, D> Drop for RcuThread<F, R, D>
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
