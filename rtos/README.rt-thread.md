
---
# [memory](https://www.rt-thread.org/document/site/programming-manual/memory/memory/)

- 最小内存管理(mem)


---
# [thread](https://www.rt-thread.org/document/site/programming-manual/thread/thread/)

每个thread创建时，会在其stack最开始构造`rt_thread_exit`函数的调用栈，当thread entry函数退出时，就会在`rt_thread_exit`中调用`rt_schedule`函数，继续调度下一个thread。

每个thread创建时，会创建一个timer，当thread的时间片结束时，会在`rt_thread_timeout`中调用`rt_schedule`函数，用于调度下一个thread。

## schedule

每个Cpu Core需要初始化一个优先级调度数组，长度为`RT_THREAD_PRIORITY_MAX`，即为优先级的支持范围，相同优先级的thread，放在同一个链表上。

 - 抢占：在`rt_schedule`中，每次调度优先级最高的thread运行；
 - 轮调：优先级相同时，会根据thread的时间片，轮调每个thread；

thread除了等待时间片结束时让出cpu外，还可以通过`sleep, yiled`等函数主动让出cpu资源。

thread让出CPU控制权有2种方式：

 - 主动：时间片结束时让出CPU，同时调度下一个thread
 - 被动：thread自己调用`sleep, yiled`等函数主动让出CPU

## timer

系统初始化时，需要在tick timer中断中调用`rt_tick_increase`函数，对`rt_current_thread`的tick计数更新，若时间片结束，则调用`rt_thread_yield`。

> 调用该函数(rt_thread_yield)后，当前线程首先把自己从它所在的就绪优先级线程队列中删除，然后把自己挂到这个优先级队列链表的尾部，然后激活调度器进行线程上下文切换（如果当前优先级只有这一个线程，则这个线程继续执行，不进行上下文切换动作）。

根据官方说明，若是一个高优先级的thread因为某种原因进入了`while(1)`，则低优先级的线程将分不得时间片。
