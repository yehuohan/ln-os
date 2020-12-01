use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    sync::atomic::{AtomicU64, Ordering},
};
use alloc::boxed::Box;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

/// TaskId作为Task唯一id，TaskId::new()生成的id需要保证唯一性。
/// NEXT_ID.fetch_add()为原子操作，可以保证NEXT_ID的自增同步。
/// （注意：需要处理u64溢出或id没有及时回收而导致Id重复问题）
impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// 一个Task包括一个唯一id和一个Future；
///
/// - Future说明：
/// Output=(): 一个task不需要返回任何结果
/// dyn Future: task可以使用何任实现Future<Output=()>的类型
/// Pin: 禁止移动内存，禁止使用可变引用(&mut)
pub struct Task {
    /// Task::id对cotask模块可见
    pub(super) id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    /// 用于Executor对Task执行poll操作（本质是对Future执行poll操作）；
    /// Task::poll对cotask模块可见。
    pub(super) fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
