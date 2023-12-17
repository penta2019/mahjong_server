use std::mem;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::task::{RawWaker, RawWakerVTable, Waker};

#[derive(Debug)]
pub struct Waiter {
    recv: Receiver<()>,
}

impl Waiter {
    pub fn wait(&self) {
        self.recv.recv().unwrap();
    }
}

pub fn waiter_waker() -> (Waiter, Waker) {
    let (sender, receiver) = channel();
    let waiter = Waiter { recv: receiver };

    // 参照カウントの管理をArcにまかせて同じsender(=data)を使い回す
    // Wakerは別スレッドから呼び出される可能性があるのでdataはスレッドセーフである必要がある
    // SenderはSendとSyncを実装しているのでこれを満たす
    let ptr = Arc::into_raw(Arc::new(sender)).cast::<()>();
    let raw_waker = RawWaker::new(ptr, waker_vtable());
    let waker = unsafe { Waker::from_raw(raw_waker) };

    (waiter, waker)
}

type D = Sender<()>;

// Virtual Table
fn waker_vtable() -> &'static RawWakerVTable {
    &RawWakerVTable::new(vte_clone, vte_wake, vte_wake_by_ref, vte_drop)
}

unsafe fn vte_clone(data: *const ()) -> RawWaker {
    // 参照カウントを増やして内部のdataを使いまわす
    // 元のArcを復元
    let arc = Arc::<D>::from_raw(data.cast::<D>());
    // Drop(=参照カウントの減少)を阻止
    let arc = mem::ManuallyDrop::new(arc);
    // 参照カウントを増やす
    let _arc_clone: mem::ManuallyDrop<_> = arc.clone();

    RawWaker::new(data, waker_vtable())
}

unsafe fn vte_wake(data: *const ()) {
    let arc = Arc::from_raw(data.cast::<D>());
    arc.send(()).unwrap();
}

unsafe fn vte_wake_by_ref(data: *const ()) {
    // 参照なので復元したArcのDropによって参照カウントが減るのを防止
    let arc = mem::ManuallyDrop::new(Arc::<D>::from_raw(data.cast::<D>()));
    arc.send(()).unwrap();
}

unsafe fn vte_drop(data: *const ()) {
    // 参照カウントを減らす
    drop(Arc::<D>::from_raw(data.cast::<D>()))
}
