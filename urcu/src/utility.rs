use std::marker::PhantomData;

#[allow(dead_code)]
pub struct UnSend<T>(*const T);
unsafe impl<T> Sync for UnSend<T> {}

#[allow(dead_code)]
pub struct UnSync<T>(*const T);
unsafe impl<T> Send for UnSync<T> {}

pub type PhantomUnsync<T = ()> = PhantomData<UnSync<T>>;

pub type PhantomUnsend<T = ()> = PhantomData<UnSend<T>>;

#[allow(dead_code)]
pub mod asserts {
    use super::*;

    pub type NotSendNotSync = (UnSend<()>, UnSync<()>);
    pub type SendButNotSync = ((), UnSync<()>);
    pub type NotSendButSync = (UnSend<()>, ());
    pub type SendAndSync = ((), ());
}
