
# CPU Exceptions

CPU发生成Exception异常时，会产生中断，例如：

- Page Fault: 发生非法访问内存时产生Page Fault
- Invalid Opcode: 执行不支持的CPU指令（instructions）
- Double Fault: CPU正要处理一个exception时，又产生一个exception
- Triple Fault: CPU正要处理Double Fault时，又产生一个exception

[x86架构的Expction类型](https://wiki.osdev.org/Exceptions)


## Interrupt Descriptor Table(IDT)

x86_64使用IDT来处理CPU产生的各中Exception；IDT数据保存在IDTR寄存器，使用lidt、sidt可以访问IDT。

对于64位架构，一个中断描述符串（Interrupt Descriptor Entry）占用16bytes，结构如下：

```
u16 | Function Pointer[0:15]
u16 | GDT selector
u16 | Options
u16 | Function Pointer[16:31]
u32 | Function Pointer[32:63]
u32 | Reserved
```

Function Pointer是Exception的Exception Handler函数（中断函数）偏移地址，和GDT用于确定函数代码地址。

产生Exception的处理过程：

- 保存现场
- 读取Exception的Index，读取对应的IDE
- 根据IDE.Options进行相关设置
- 加载IDE.GDT到CS寄存器
- 跳转到Funtion Pointer执行

## Calling Convention（调用约定）

### extern "C"

"C"是C函数调用约定（详见[System V ABI](https://wiki.osdev.org/System_V_ABI)）

- 传递函数前6个整型参数的寄存器：rdi, rsi, rdx, rcx, r8, r9
- 传递其余的参数寄存器：stack
- 返回结果寄存器：rax, rdx
- Preserved寄存器（由被调用的函数callee保存）：rbp, rbx, rsp, r12, r13, r14, r15
- Scratch寄存器（由调用callee函数的caller保存）：rax, rcx, rdx, rsi, rdi, r8, r9, r10
- 调用函数指令：`call`
- 函数返回指令：`ret`


### extern "x86-interrupt"

"x86-interrupt"调用约定，可以保证在Exception Hanlder函数返回时，CPU所有的寄存器都设回了原来的值。
（lnos中使用extern "x86-interrupt"）。


## Interrupt Stack Frame

![exception stack frame](img/exception-stack-frame.svg)

在调用中断函数前，需要保存Stack(ss, rsp)、rflags、Code(ss, rip)等。

> 当CPU执行特权级别切换时（例如用户空调和内核空间的切换），需要切换Stack


### Double Fault

Kernel栈溢出时，未能正确处理Double Fault，而导致Triple Fault的过程示例：

> A guard page is a special memory page at the bottom of a stack that makes it possible to detect stack overflows.

1. 系统发生栈溢出时，会最终调用到Guard Page，但是Guard Page没有映射到实际的物理地址；

2. 读取Guard Page会触发Page Fault，之后会将读取中断栈帧入栈，而此时栈寄存器仍指向Guard Page，从而又触发Page Fault，即Double Fault；

3. 之后就会调用Double Fault Handler，但是调用前又要将栈寄存器指向的Guard Page入栈，从而触发Triple Fault，然后CPU重启。

x86_64可以提前设置好可以正常使用的中断栈表IST(Interrupt Stack Table)，当发生Exception时，可以先切换一个中断栈帧（设置Options[0:2]来选择IST的Index），这样就可以顺利调用Double Fault Handler，从而避免发生Triple Fault。

> IST是用于64位模式下，是TSS(Task State Segment)的一部分，而TSS可以兼容32位模式的处理。


# Hardware Interrupts

CPU通过PIC中断控器（Programmable Interrupt Controller，现在已经被APIC, Advanced Programmable Interrupt Controller取代），来管理时钟、键盘等各种硬件中断。

CPU通过IO端口配置PIC：

- Primary PIC: 0x20 (command) and 0x21 (data)
- Secondary PIC: 0xa0 (command) and 0xa1 (data)

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

- 死锁问题

vag::VGA使用了spin::lock，当_start中使用VGA打印时，若Timer中断也使用VGA打印，则会造成死锁。
（除非是多核，_start在一个核中运行，而Timer中断使用另一个核处理）

解决方法：使用VGA时关闭中断。
