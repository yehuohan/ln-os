//! 协作式多任务处理

pub mod task;
pub mod executor;

pub fn run() {
    let mut executor = executor::Executor::new();
    executor.spawn(task::Task::new(first_task()));
    executor.run();
}

async fn first_task() {
    println!("start task schedule");
}
