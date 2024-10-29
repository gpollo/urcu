//! This module implements a cleaner thread.
//!
//! The goal is to allow any thread (registered or not) to execute
//! a callback on a registered thread. It is currently only used for
//! cleaning up [`RcuRef`].
//!
//! [`RcuRef`]: crate::rcu::reference::RcuRef

use std::sync::mpsc::{Receiver, Sender};
use std::sync::RwLock;
use std::thread::JoinHandle;

use super::RcuContext;

/// Defines the cleanup callback signature.
pub type RcuCleanup<C> = Box<dyn FnOnce(&C) + Send + 'static>;

/// Defines the cleanup callback signature.
pub type RcuCleanupMut<C> = Box<dyn FnOnce(&mut C) + Send + 'static>;

enum RcuCleanerCommand<C> {
    Execute(RcuCleanup<C>),
    ExecuteMut(RcuCleanupMut<C>),
    Barrier(Sender<()>),
    Shutdown,
}

struct RcuCleanerThread<C> {
    commands: Receiver<RcuCleanerCommand<C>>,
}

impl<C> RcuCleanerThread<C>
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
                Ok(RcuCleanerCommand::Shutdown) => break,
                Ok(RcuCleanerCommand::Barrier(sender)) => {
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

struct RcuCleanupHandle<C> {
    thread: Option<JoinHandle<()>>,
    callbacks: Sender<RcuCleanerCommand<C>>,
}

impl<C> RcuCleanupHandle<C>
where
    C: RcuContext + 'static,
{
    fn try_get(instance: &RwLock<Option<Self>>) -> Option<RcuCleanupSender<C>> {
        instance
            .read()
            .unwrap()
            .as_ref()
            .map(|handle| RcuCleanupSender(handle.callbacks.clone()))
    }

    fn set(instance: &RwLock<Option<Self>>) -> RcuCleanupSender<C> {
        RcuCleanupSender(
            instance
                .write()
                .unwrap()
                .get_or_insert_with(|| {
                    let (tx, rx) = std::sync::mpsc::channel();

                    Self {
                        thread: Some(RcuCleanerThread::start(rx)),
                        callbacks: tx,
                    }
                })
                .callbacks
                .clone(),
        )
    }

    fn get(instance: &RwLock<Option<Self>>) -> RcuCleanupSender<C> {
        Self::try_get(instance).unwrap_or_else(|| Self::set(instance))
    }

    fn delete(instance: &RwLock<Option<Self>>) {
        if let Some(handle) = instance.write().unwrap().take().and_then(|instance| {
            instance
                .callbacks
                .send(RcuCleanerCommand::Shutdown)
                .unwrap();
            instance.thread
        }) {
            if let Err(e) = handle.join() {
                log::error!("failed to join cleanup thread: {:?}", e);
            }
        }
    }
}

pub struct RcuCleanupSender<C>(Sender<RcuCleanerCommand<C>>);

impl<C> RcuCleanupSender<C> {
    pub fn send(&self, callback: RcuCleanup<C>) {
        let command = RcuCleanerCommand::Execute(callback);
        if let Err(e) = self.0.send(command) {
            log::error!("failed to send execute command: {:?}", e);
        }
    }

    pub fn send_mut(&self, callback: RcuCleanupMut<C>) {
        let command = RcuCleanerCommand::ExecuteMut(callback);
        if let Err(e) = self.0.send(command) {
            log::error!("failed to send execute command: {:?}", e);
        }
    }

    pub fn barrier(&self) {
        let (tx, rx) = std::sync::mpsc::channel();

        let command = RcuCleanerCommand::Barrier(tx);
        if let Err(e) = self.0.send(command) {
            log::error!("failed to send barrier command: {:?}", e);
        } else if let Err(e) = rx.recv() {
            log::error!("failed to wait for barrier: {:?}", e);
        }
    }
}

#[cfg(feature = "flavor-bp")]
mod bp {
    use super::*;

    use crate::rcu::flavor::RcuContextBp;

    static INSTANCE: RwLock<Option<RcuCleanupHandle<RcuContextBp>>> = RwLock::new(None);

    impl RcuCleanupSender<RcuContextBp> {
        pub fn get() -> Self {
            RcuCleanupHandle::<RcuContextBp>::get(&INSTANCE)
        }

        pub fn delete() {
            RcuCleanupHandle::<RcuContextBp>::delete(&INSTANCE)
        }
    }
}

#[cfg(feature = "flavor-mb")]
mod mb {
    use super::*;

    use crate::rcu::flavor::RcuContextMb;

    static INSTANCE: RwLock<Option<RcuCleanupHandle<RcuContextMb>>> = RwLock::new(None);

    impl RcuCleanupSender<RcuContextMb> {
        pub fn get() -> Self {
            RcuCleanupHandle::<RcuContextMb>::get(&INSTANCE)
        }

        pub fn delete() {
            RcuCleanupHandle::<RcuContextMb>::delete(&INSTANCE)
        }
    }
}

#[cfg(feature = "flavor-memb")]
mod memb {
    use super::*;

    use crate::rcu::flavor::RcuContextMemb;

    static INSTANCE: RwLock<Option<RcuCleanupHandle<RcuContextMemb>>> = RwLock::new(None);

    impl RcuCleanupSender<RcuContextMemb> {
        pub fn get() -> Self {
            RcuCleanupHandle::<RcuContextMemb>::get(&INSTANCE)
        }

        pub fn delete() {
            RcuCleanupHandle::<RcuContextMemb>::delete(&INSTANCE)
        }
    }
}

#[cfg(feature = "flavor-qsbr")]
mod qsbr {
    use super::*;

    use crate::rcu::flavor::RcuContextQsbr;

    static INSTANCE: RwLock<Option<RcuCleanupHandle<RcuContextQsbr>>> = RwLock::new(None);

    impl RcuCleanupSender<RcuContextQsbr> {
        pub fn get() -> Self {
            RcuCleanupHandle::<RcuContextQsbr>::get(&INSTANCE)
        }

        pub fn delete() {
            RcuCleanupHandle::<RcuContextQsbr>::delete(&INSTANCE)
        }
    }
}
