
# Task

## [Task implemntation](https://www.freertos.org/implementation/main.html)

- Context

Processor registers, stack, etc(resource may accessed or modified commonly by other task) are so called task **context**.
Saving context of a task being suspended, and restoring context of a task being resumed is so called **context switching**.

- Idle task

Executed when no other running task. Can simply take as the code `while (1) {}` of `main()`.

- Tick task

Executing ISR function(tick interrupt) with a tick time, which will control switching to context of time-ready user task.

- Preemptive

```
Priority      save context(idle's)
                      \      restore context(user's), then 'reti' will return to User Task's function.
                       \     /
high    Tick Task       -----      switch to idle task
 |                                /
 |      User Task            -----
 v
log     Idle Task   ----          ----------------
```

## Communication

 - `Queue`: Blocking when attempt to read empty queue or write full queue. `critical` is required.


# Ports

## Arm Cortex-M(stm32)

- `xPortSysTickHandler`: Check that if there is task required to switch, trigger the `xPortPendSVHandler` interrupt.
- `xPortPendSVHandler`: Switching to context of ready task.
- `Critical`: Enter with disable interrupt and exit with enable interrupt.
- `Semaphore`: Implementation via queue. Semaphore is a queue with an item size of 0, and where the number of messages in the queue is the semaphore's count value.
- `Mutex`: Use binary semaphore.
- `xxxFromISR`: Function called from interrupt function.

