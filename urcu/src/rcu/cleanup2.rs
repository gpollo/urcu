//! This module implements a cleaner thread.
//!
//! The goal is to allow any thread (registered or not) to execute
//! a callback on a registered thread. It is currently only used for
//! cleaning up [`RcuRef`].
//!
//! [`RcuRef`]: crate::rcu::reference::RcuRef

use std::cell::{Cell, OnceCell};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock, Weak};
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
                Ok(RcuCleanerCommand::Shutdown) | Err(_) => {
                    log::debug!("shutting down RCU cleanup thread");
                    break;
                }
            }
        }
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

    pub fn get(instance: &RwLock<Option<Self>>) -> RcuCleanupSender<C> {
        Self::try_get(instance).unwrap_or_else(|| Self::set(instance))
    }

    pub fn delete(instance: &RwLock<Option<Self>>) {
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

struct RcuCleanupSender<C>(Sender<RcuCleanerCommand<C>>);

impl<C> RcuCleanupSender<C> {
    pub fn send(&self, callback: RcuCleanup<C>) {
        if self.0.send(RcuCleanerCommand::Execute(callback)).is_err() {
            log::error!("failed to send cleanup execute command");
        }
    }

    pub fn send_mut(&self, callback: RcuCleanupMut<C>) {
        if self
            .0
            .send(RcuCleanerCommand::ExecuteMut(callback))
            .is_err()
        {
            log::error!("failed to send cleanup execute command");
        }
    }
}

// impl<C> RcuCleanupSender<C> {

//     pub fn remove(&self) {
//         // The last thread doing this will join the cleanup thread.
//         self.thread.set(None);
//     }
// }

// macro_rules! impl_cleanup_for_context {
//     ($context:ident) => {
//         static CLEANUP_THREAD: Mutex<Weak<RcuCleanupThread<$context>>> = Mutex::new(Weak::new());

//         impl $context {
//             thread_local! {
//                 static CLEANUP_SENDER: OnceCell<RcuCleanupSender<$context>> = OnceCell::new();
//             }

//             pub(crate) fn cleanup_send(callback: RcuCleanupMut<Self>) {
//                 Self::CLEANUP_SENDER.with(|cell| {
//                     cell.get_or_init(|| RcuCleanupThread::get(&CLEANUP_THREAD))
//                         .send_mut(callback);
//                 });
//             }

//             pub(crate) fn cleanup_send_and_block(callback: RcuCleanup<Self>) {
//                 Self::CLEANUP_SENDER.with(|cell| {
//                     let (tx, rx) = std::sync::mpsc::channel::<()>();

//                     cell.get_or_init(|| RcuCleanupThread::get(&CLEANUP_THREAD))
//                         .send(Box::new(move |mut context| {
//                             callback(&mut context);
//                             if let Err(e) = tx.send(()) {
//                                 log::error!("failed to send cleanup signal: {:?}", e);
//                             }
//                         }));

//                     if let Err(e) = rx.recv() {
//                         log::error!("failed to receive cleanup signal: {:?}", e);
//                     }
//                 });
//             }

//             pub(crate) fn cleanup_remove() {
//                 Self::CLEANUP_SENDER.with(|cell| {
//                     if let Some(sender) = cell.get() {
//                         sender.remove();
//                     }
//                 });
//             }
//         }
//     };
// }

// #[cfg(feature = "flavor-bp")]
// mod bp {
//     use super::*;

//     use crate::rcu::flavor::RcuContextBp;

//     impl_cleanup_for_context!(RcuContextBp);
// }

// #[cfg(feature = "flavor-mb")]
// mod mb {
//     use super::*;

//     use crate::rcu::flavor::RcuContextMb;

//     impl_cleanup_for_context!(RcuContextMb);
// }

// #[cfg(feature = "flavor-memb")]
// mod memb {
//     use super::*;

//     use crate::rcu::flavor::RcuContextMemb;

//     impl_cleanup_for_context!(RcuContextMemb);
// }

// #[cfg(feature = "flavor-qsbr")]
// mod qsbr {
//     use super::*;

//     use crate::rcu::flavor::RcuContextQsbr;

//     impl_cleanup_for_context!(RcuContextQsbr);
// }
