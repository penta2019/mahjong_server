use std::mem;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::task::{RawWaker, RawWakerVTable, Waker};
use std::time::Duration;

use crate::util::misc::Res;

#[derive(Debug)]
pub struct Waiter {
    recv: Receiver<()>,
}

impl Waiter {
    pub fn wait(&self) {
        self.recv.recv().unwrap();
    }

    pub fn wait_timeout(&self, timeout: Duration) -> Res {
        Ok(self.recv.recv_timeout(timeout)?)
    }
}

pub fn waiter_waker() -> (Waiter, Waker) {
    let (sender, receiver) = channel();
    let waiter = Waiter { recv: receiver };

    // Wakerは別スレッドから呼び出される可能性があるのでdataはスレッドセーフである必要がある
    // SenderはSendとSyncを実装しているのでこれを満たす
    let ptr = Arc::into_raw(Arc::new(sender)).cast::<()>();
    let raw_waker = RawWaker::new(ptr, RAW_WAKER_VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };

    (waiter, waker)
}

type S = Sender<()>;

// Virtual Table
static RAW_WAKER_VTABLE: &RawWakerVTable =
    &RawWakerVTable::new(vte_clone, vte_wake, vte_wake_by_ref, vte_drop);

unsafe fn vte_clone(data: *const ()) -> RawWaker {
    let arc = unsafe { Arc::<S>::from_raw(data.cast::<S>()) };
    let ptr = Arc::into_raw(arc.clone()).cast::<()>();
    let _ = mem::ManuallyDrop::new(arc); // Dropを阻止
    RawWaker::new(ptr, RAW_WAKER_VTABLE)
}

unsafe fn vte_wake(data: *const ()) {
    let arc = unsafe { Arc::from_raw(data.cast::<S>()) };
    arc.send(()).unwrap();
}

unsafe fn vte_wake_by_ref(data: *const ()) {
    let arc = mem::ManuallyDrop::new(unsafe { Arc::<S>::from_raw(data.cast::<S>()) });
    arc.send(()).unwrap();
}

unsafe fn vte_drop(data: *const ()) {
    drop(unsafe { Arc::<S>::from_raw(data.cast::<S>()) });
}
