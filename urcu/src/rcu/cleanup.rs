//! This module implements a cleaner thread.
//!
//! The goal is to allow any thread (registered or not) to execute
//! a callback on a registered thread. It is currently only used for
//! cleaning up [`RcuRef`].
//!
//! [`RcuRef`]: crate::rcu::reference::RcuRef

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Once, RwLock};
use std::thread::JoinHandle;

use super::RcuContext;

/// Defines the cleanup callback signature.
pub type RcuCleanup<C> = Box<dyn FnOnce(&C) + Send + 'static>;

/// Defines the cleanup callback signature.
pub type RcuCleanupMut<C> = Box<dyn FnOnce(&mut C) + Send + 'static>;

type ContextFn<C> = Box<dyn FnOnce() -> C + Send>;

enum Command<C> {
    Execute(RcuCleanup<C>),
    ExecuteMut(RcuCleanupMut<C>),
    Barrier(Sender<()>),
    Shutdown,
}

struct Thread<C> {
    commands: Receiver<Command<C>>,
}

impl<C> Thread<C>
where
    C: 'static,
{
    fn start(context: ContextFn<C>, commands: Receiver<Command<C>>) -> JoinHandle<()> {
        std::thread::Builder::new()
            .name(format!(
                "urcu::cleanup::{}",
                std::any::type_name::<C>()
                    .split("::")
                    .last()
                    .unwrap()
                    .replace("RcuContext", "")
                    .to_lowercase()
            ))
            .spawn(move || Self { commands }.run(context))
            .unwrap()
    }

    fn run(self, context: ContextFn<C>) {
        log::debug!("launching cleanup thread");

        let mut context = context();

        loop {
            match self.commands.recv() {
                Ok(Command::Execute(callback)) => callback(&context),
                Ok(Command::ExecuteMut(callback)) => callback(&mut context),
                Ok(Command::Shutdown) => break,
                Ok(Command::Barrier(sender)) => {
                    if let Err(e) = sender.send(()) {
                        log::error!("failed to execute cleanup barrier: {:?}", e);
                    }
                }
                Err(e) => {
                    log::error!("failed to get cleanup command: {:?}", e);
                    break;
                }
            }
        }

        log::debug!("shutting down cleanup thread");
    }
}

struct ThreadHandle<C> {
    thread: Option<JoinHandle<()>>,
    callbacks: Sender<Command<C>>,
}

impl<C> ThreadHandle<C>
where
    C: RcuContext + 'static,
{
    fn create(instance: &RwLock<Option<Self>>, context: ContextFn<C>) -> RcuCleaner<C> {
        RcuCleaner(
            instance
                .write()
                .unwrap()
                .get_or_insert_with(|| {
                    let (tx, rx) = std::sync::mpsc::channel();

                    Self {
                        thread: Some(Thread::start(context, rx)),
                        callbacks: tx,
                    }
                })
                .callbacks
                .clone(),
        )
    }

    fn try_get(instance: &RwLock<Option<Self>>) -> Option<RcuCleaner<C>> {
        instance
            .read()
            .unwrap()
            .as_ref()
            .map(|handle| RcuCleaner(handle.callbacks.clone()))
    }

    fn get(instance: &RwLock<Option<Self>>, context: ContextFn<C>) -> RcuCleaner<C> {
        Self::try_get(instance).unwrap_or_else(|| Self::create(instance, context))
    }

    fn delete(instance: &RwLock<Option<Self>>) {
        instance.write().unwrap().take();
    }
}

impl<C> Drop for ThreadHandle<C> {
    fn drop(&mut self) {
        log::trace!("sending shutdown command");

        if let Err(e) = self.callbacks.send(Command::Shutdown) {
            log::error!("failed to send shutdown command: {:?}", e);
            return;
        }

        if let Some(handle) = self.thread.take() {
            if let Err(e) = handle.join() {
                log::error!("failed to join cleanup thread: {:?}", e);
            }
        }
    }
}

pub struct RcuCleaner<C>(Sender<Command<C>>);

impl<C> RcuCleaner<C> {
    pub fn send(&self, callback: RcuCleanup<C>) -> &Self {
        let command = Command::Execute(callback);
        if let Err(e) = self.0.send(command) {
            log::error!("failed to send execute command: {:?}", e);
        }

        self
    }

    pub fn send_mut(&self, callback: RcuCleanupMut<C>) -> &Self {
        let command = Command::ExecuteMut(callback);
        if let Err(e) = self.0.send(command) {
            log::error!("failed to send execute command: {:?}", e);
        }

        self
    }

    pub fn barrier(&self) -> &Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let command = Command::Barrier(tx);
        if let Err(e) = self.0.send(command) {
            log::error!("failed to send barrier command: {:?}", e);
        } else if let Err(e) = rx.recv() {
            log::error!("failed to wait for barrier: {:?}", e);
        } else {
            log::trace!("finished barrier command");
        }

        self
    }
}

macro_rules! impl_cleanup_for_context {
    ($context:ident) => {
        use crate::rcu::flavor::$context;

        static REGISTER_ATEXIT: Once = Once::new();
        static INSTANCE: RwLock<Option<ThreadHandle<$context>>> = RwLock::new(None);

        impl RcuCleaner<$context> {
            extern "C" fn delete() {
                ThreadHandle::<$context>::delete(&INSTANCE);
            }

            pub fn get() -> Self {
                REGISTER_ATEXIT.call_once(|| unsafe {
                    assert_eq!(libc::atexit(Self::delete), 0);
                });

                let context = Box::new(|| $context::rcu_register().unwrap());
                ThreadHandle::<$context>::get(&INSTANCE, context)
            }
        }
    };
}

#[cfg(feature = "flavor-bp")]
mod bp {
    use super::*;

    impl_cleanup_for_context!(RcuContextBp);
}

#[cfg(feature = "flavor-mb")]
mod mb {
    use super::*;

    impl_cleanup_for_context!(RcuContextMb);
}

#[cfg(feature = "flavor-memb")]
mod memb {
    use super::*;

    impl_cleanup_for_context!(RcuContextMemb);
}

#[cfg(feature = "flavor-qsbr")]
mod qsbr {
    use super::*;

    impl_cleanup_for_context!(RcuContextQsbr);
}
