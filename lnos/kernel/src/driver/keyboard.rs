//! 按键处理模块

use core::{
    pin::Pin,
    task::{Poll, Context},
};
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};


/// 使用ArrayQueue作为键盘的键码数据流队列；
/// ArrayQeueu是无锁队列，队列长度在初始化时给定（类似于RingBuffer）。
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

static WAKER: AtomicWaker = AtomicWaker::new();

/// 从键盘按键队列中取出scancode流（异步取出）
pub struct ScancodeStream;

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    /// poll_next可以一直从队列中重复的读取scancode，直至返回Poll::Ready(None)表示数据流已经结束。
    /// 即没有scancode返回Poll::Pending，有scandcode返回Poll:Ready(Some(scancode))，
    /// 不再读取按键的scancode则返回Poll::Ready(None)。
    /// 
    /// 为什么不用Futrue？
    /// 若有scancode就需要返回Poll::Ready，而Future返回Ready就表示按键处理任务结束了，不适合scancode数据流
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        } // 第一次检查队列，有scancode则返回Ready

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take(); // 第二次检查队列，发现有了新的scancode，需要移除waker notification
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}


/// 往键盘按键队列中添加scancode；
/// 只在crate-lib中可见（只用于键盘中断中缓存scancode）。
pub(crate) fn append_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full");
        } else {
            WAKER.wake(); // 队列中有新的scancode，通知executor处理
        }
    } else {
        println!("WARNING: scancode queu uninitialized")
    }
}

/// 键盘按键处理Task
pub async fn task_keyboard() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        // 解码scancode并处理
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
