
# rcore(x86_64)

基于Rust开发的x86_64 OS。


# VGA Display

对`0xb8000`地址进行读写，实现控制台的基本输入输出。


# CPU Exceptions

需要初始化IDT，实现中断和异常处理。

## 中断调用约定

- 传递前6个整型参数的寄存器：rdi, rsi, rdx, rcx, r8, r9
- 其余的参数：stack
- 返回结果：rax, rdx

## 相关寄存器

| Preserved (callee-saved)          | Scratch (caller-saved)               |
| -                                 | -                                    |
| rbp, rbx, rsp, r12, r13, r14, r15 | rax, rcx, rdx, rsi, rdi, r8, r9, r10 |

- callee: 被调用的函数
- caller: 调用callee的函数


## 中断栈帧

- 正常函数调用： 正常函数调用使用`call`指令，返回使用`ret`指令。

## 双重异常(Double Fault)

系统发生栈溢出时，会最终调用到Guard Page，但是Guard Page没有映射到实际的物理地址；
读取Guard Page会触发Page Fault，之后会将读取中断栈帧入栈，而此时栈寄存器仍指向Guard Page，从而又触发Page Fault，即Double Fault；
之后就会调用Double Fault Handler，但是调用前又要将栈寄存器指向的Guard Page入栈，从而触发Triple Fault，然后CPU重启。

为了防止Triple Fault，可以设置IST(Interrupt Stack Table)，在发生Double Fault时，CPU自动将栈寄存器指向IST，这样就可以顺利调用Double Fault Handler了。

IST是用于64位模式下，是TSS(Task State Segment)的一部分，而TSS可以兼容32位模式的处理。


# 硬件中断（Hardware Interrupts）

CPU通过中断控器，来管理时钟、键盘等中断。

```
                     ____________                          ____________
Real Time Clock --> |            |   Timer -------------> |            |
ACPI -------------> |            |   Keyboard-----------> |            |      _____
Available --------> | Secondary  |----------------------> | Primary    |     |     |
Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
Primary ATA ------> |            |   Floppy disk -------> |            |
Secondary ATA ----> |____________|   Parallel Port 1----> |____________|
```

## 死锁问题（Deadlocks）

vag中的WRITER使用spinlock，当main中使用WRITER打印时，若Timer中断也使用WRITER打印，则会造成死锁。
（除非是多核，main中1个核中运行，而Timer中断使用另一个核处理）

- 解决方法：使用WRITER时关闭中断
