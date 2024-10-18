//! This module implements a cleaner thread.
//!
//! The goal is to allow any thread (registered or not) to execute
//! a callback on a registered thread. It is currently only used for
//! cleaning up [`RcuRef`].
//!
//! [`RcuRef`]: crate::rcu::reference::RcuRef

use std::cell::{Cell, OnceCell};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, Weak};
use std::thread::JoinHandle;

use super::RcuContext;

/// Defines the cleanup callback signature.
pub type RcuCleanup<C> = Box<dyn FnOnce(&C) + Send + 'static>;

/// Defines the cleanup callback signature.
pub type RcuCleanupMut<C> = Box<dyn FnOnce(&mut C) + Send + 'static>;

enum RcuCleanerCommand<C> {
    Execute(RcuCleanup<C>),
    ExecuteMut(RcuCleanupMut<C>),
    Shutdown,
}

struct RcuCleaner<C> {
    commands: Receiver<RcuCleanerCommand<C>>,
}

impl<C> RcuCleaner<C>
where
    C: RcuContext + 'static,
{
    fn start(commands: Receiver<RcuCleanerCommand<C>>) -> JoinHandle<()> {
        std::thread::spawn(|| Self { commands }.run())
    }

    fn run(self) {
        let mut context = C::rcu_register().unwrap();

        loop {
            match self.commands.recv() {
                Ok(RcuCleanerCommand::Execute(callback)) => callback(&context),
                Ok(RcuCleanerCommand::ExecuteMut(callback)) => callback(&mut context),
                Ok(RcuCleanerCommand::Shutdown) | Err(_) => {
                    println!("shutting down RCU cleanup thread");
                    break;
                }
            }
        }
    }
}

struct RcuCleanupThread<C> {
    thread: Option<JoinHandle<()>>,
    callbacks: Sender<RcuCleanerCommand<C>>,
}

impl<C> RcuCleanupThread<C>
where
    C: RcuContext + 'static,
{
    fn new() -> Arc<Self> {
        let (tx, rx) = std::sync::mpsc::channel();

        Arc::new(Self {
            thread: Some(RcuCleaner::start(rx)),
            callbacks: tx,
        })
    }

    pub fn get(mutex_ptr: &Mutex<Weak<Self>>) -> RcuCleanupSender<C> {
        let mut weak_ptr = mutex_ptr.lock().unwrap();
        let arc_ptr = if let Some(arc_ptr) = weak_ptr.upgrade() {
            arc_ptr
        } else {
            let arc_ptr = Self::new();
            *weak_ptr = Arc::<RcuCleanupThread<C>>::downgrade(&arc_ptr);
            arc_ptr
        };

        RcuCleanupSender {
            thread: Cell::new(Some(arc_ptr.clone())),
            callbacks: arc_ptr.callbacks.clone(),
        }
    }
}

impl<C> Drop for RcuCleanupThread<C> {
    fn drop(&mut self) {
        if self.callbacks.send(RcuCleanerCommand::Shutdown).is_err() {
            log::error!("failed to send cleanup shutdown command");
        }

        match self.thread.take().map(|t| t.join()) {
            None => (),
            Some(Ok(_)) => (),
            Some(Err(e)) => log::error!("failed to join cleanup thread {:?}", e),
        }
    }
}

struct RcuCleanupSender<C> {
    thread: Cell<Option<Arc<RcuCleanupThread<C>>>>,
    callbacks: Sender<RcuCleanerCommand<C>>,
}

impl<C> RcuCleanupSender<C> {
    pub fn send(&self, callback: RcuCleanup<C>) {
        if self
            .callbacks
            .send(RcuCleanerCommand::Execute(callback))
            .is_err()
        {
            log::error!("failed to send cleanup execute command");
        }
    }

    pub fn send_mut(&self, callback: RcuCleanupMut<C>) {
        if self
            .callbacks
            .send(RcuCleanerCommand::ExecuteMut(callback))
            .is_err()
        {
            log::error!("failed to send cleanup execute command");
        }
    }

    pub fn remove(&self) {
        // The last thread doing this will join the cleanup thread.
        self.thread.set(None);
    }
}

macro_rules! impl_cleanup_for_context {
    ($context:ident) => {
        static CLEANUP_THREAD: Mutex<Weak<RcuCleanupThread<$context>>> = Mutex::new(Weak::new());

        impl $context {
            thread_local! {
                static CLEANUP_SENDER: OnceCell<RcuCleanupSender<$context>> = OnceCell::new();
            }

            pub(crate) fn cleanup_send(callback: RcuCleanupMut<Self>) {
                Self::CLEANUP_SENDER.with(|cell| {
                    cell.get_or_init(|| RcuCleanupThread::get(&CLEANUP_THREAD))
                        .send_mut(callback);
                });
            }

            pub(crate) fn cleanup_send_and_block(callback: RcuCleanup<Self>) {
                Self::CLEANUP_SENDER.with(|cell| {
                    let (tx, rx) = std::sync::mpsc::channel::<()>();

                    cell.get_or_init(|| RcuCleanupThread::get(&CLEANUP_THREAD))
                        .send(Box::new(move |mut context| {
                            callback(&mut context);
                            if let Err(e) = tx.send(()) {
                                log::error!("failed to send cleanup signal: {:?}", e);
                            }
                        }));

                    if let Err(e) = rx.recv() {
                        log::error!("failed to receive cleanup signal: {:?}", e);
                    }
                });
            }

            pub(crate) fn cleanup_remove() {
                Self::CLEANUP_SENDER.with(|cell| {
                    if let Some(sender) = cell.get() {
                        sender.remove();
                    }
                });
            }
        }
    };
}

#[cfg(feature = "flavor-bp")]
mod bp {
    use super::*;

    use crate::rcu::flavor::RcuContextBp;

    impl_cleanup_for_context!(RcuContextBp);
}

#[cfg(feature = "flavor-mb")]
mod mb {
    use super::*;

    use crate::rcu::flavor::RcuContextMb;

    impl_cleanup_for_context!(RcuContextMb);
}

#[cfg(feature = "flavor-memb")]
mod memb {
    use super::*;

    use crate::rcu::flavor::RcuContextMemb;

    impl_cleanup_for_context!(RcuContextMemb);
}

#[cfg(feature = "flavor-qsbr")]
mod qsbr {
    use super::*;

    use crate::rcu::flavor::RcuContextQsbr;

    impl_cleanup_for_context!(RcuContextQsbr);
}
