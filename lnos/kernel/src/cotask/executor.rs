use super::task::{Task, TaskId};
use alloc::{
    collections::BTreeMap,
    sync::Arc,
    task::Wake,
};
use core::task::{
    Waker, Context, Poll,
};
use crossbeam_queue::ArrayQueue;


/// Executor是系统所有Task的调度器；
///
/// - tasks: 保存系统所有的task实例，由id索引
/// - wakers: 保存task的waker，由id索引
/// - task_queue: waker将id放入queue并且唤醒task，executor从queue取出id并执行对应的task
pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    wakers: BTreeMap<TaskId, Waker>,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

/// TaskWaker用于唤醒task，本质是通知Executor执行queue中的task。
struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            wakers: BTreeMap::new(),
        }
    }

    /// 添加task，并开始调度
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("({:?}) already in tasks", task_id);
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    /// 调度处于ready状态的task（即放置在task_queue中的task）
    fn run_ready_tasks(&mut self) {
        // 解构self成员，每个成员有各自的&mut
        let Self { tasks, task_queue, wakers } = self;

        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task, // 取对id对应的task
                None => continue, // task_id没有对应的task
            };

            let waker = wakers
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));

            // poll task所需要的上下文，即是task的waker
            match task.poll(&mut Context::from_waker(waker)) {
                Poll::Ready(()) => { // 移除执行完毕的task和waker
                    tasks.remove(&task_id);
                    wakers.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            // queue中有task就执行task，无task则处理idle状态
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        // 防止刚检查完is_empty()，立马来一个中断，导致不能及时响应
        interrupts::disable();
        if self.task_queue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

impl TaskWaker {
    /// 这里的task_queue即是Executor::task_queue，TaskWaker将id放入queue中，
    /// executor自然会执行对应的task，即实现了task的唤醒操作。
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    /// 唤醒task，即将id放入queue中；
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

/// 实现了Wake Trait，能才可唤醒Executor中的task；
/// 唤醒操作即是TaksWaker::wake_task()。
impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
