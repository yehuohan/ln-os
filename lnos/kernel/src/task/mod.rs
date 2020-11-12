
use core::{future::Future, pin::Pin};
use core::task::{Context, Poll};
use core::sync::atomic::{AtomicU64, Ordering};
use alloc::boxed::Box;

pub mod simple_executor;
pub mod executor;
pub mod keyboard;

pub struct Task {
    id: TaskId,
    /// Output=(): 一个task不需要返回任何结果
    /// dyn Future: task可以包含何任类型的Future
    /// Pin: 禁止移动内存，禁止使用可变引用(&mut)
    future: Pin<Box<dyn Future<Output = ()>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
