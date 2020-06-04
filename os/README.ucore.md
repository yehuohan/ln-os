
---
# ucore OS(x86) learning

- code from [ucore os lab](https://github.com/chyyuu/ucore_os_lab).
- docs from [ucore os docs](https://chyyuu.gitbooks.io/ucore_os_docs), [simple os book](https://chyyuu.gitbooks.io/simple_os_book/)
- x86: intel 64 位和 IA-32 架构软件开发人员手册

---
# boot

## ia32

### 实模式：使用20位地址总线，寻址空间为1Mb，使用段寄存器Seg(CS,DS,SS等)和偏移Ofs寻址

实模式下使用8086中的段偏移寻址：

```
逻辑地址Ofs : 程序中看到的地址为逻辑地址，如'int* p = &val'中的指针p
段地址Seg   : 即段寄存器中保存的值
物理地址PA  : Seg << 4 + Ofs
```

### 保护模式：使用32位地址总线，寻址空间为4GB，使用段寄存器Seg(CS,DS,SS等，也叫段选择子)、偏移Ofs和GDT寻址

保护模式下使用分段和分页机制寻址，分页可以用于内存管理：

- 无分页：逻辑地址 -> 段机制 -> 线性地址 -> 物理地址

```
逻辑地址Ofs  : 程序中看到的地址为逻辑地址，如'int* p = &val'中的指针p
段寄存器GDTR : 保存GDT数组的物理地址
段描述符GDT  : GDT.Base保存线性地址的基地址
段选择子Seg  : Seg[15:3]作为下标用于索引GDT，获取基地址GDT.Base

   ·--------- Seg:Ofs ------------·
   |                              |
   |    Descriptor Table          |
   |    ·---------------·         |
   |    |               |         |
   |    |---------------|  Base   |         (LA)
   ·--> |Seg Descriptor | ------> + ---> Base:Ofs
        |---------------|
        |               |
        ·---------------·

线性地址 LA : GDT[Seg].Base + Ofs
物理地址 PA : 无分页时，物理地址PA=线性地址LA

当GDT[Seg].Base = 0时，则有 逻辑地址 = 线性地址
```

- 有分页：逻辑地址 -> 段机制 -> 线性地址 -> 页机制 -> 物理地址

物理页按4KB对齐，所有物理页的地址的最低12位均为0，所有物理页只需要其高20位，再加上偏移，即得到一个32位完整物理地址。

```
控制寄存器CR3 : CR3[31:12]为保存页目录表PDT（Page Directory Table）的物理地址，PDT是元素为PDE（Page Directory Entry）的数组
页目录表PDE   : PDE[31:12]为保存页表PT（Page Table）的物理地址，PT是一个元素为PTE（Page Table Entry）的数组
物理页PTE     : PTE[31:12]为保存物理页地址的高20位
物理地址PA    :
    PDX  = LA[31:22] 为CR3索引  ->  PDE = CR3[31:22][PDX]
    PTX  = LA[21:12] 为PDE索引  ->  PTE = PDE[31:12][PTX]
    POfs = LA[11:0] 为偏移      ->  PA  = PTE[31:12] + POfs
    其中CR3,PDE,PTE均只用高20位来保存物理地址，低12位用于保存相关属性
```

## BootLoader

- BIOS启动

```
         Disk                 Memory
     ·----------·          ·----------·4GB
     |          |          |          |
     |          |          |          |
     |          |     ---> |Kernel    |
     |----------|    /     |----------|0x100000(1MB)
     |Kernel    | -------> |elfhdr    |
     |----------|          |----------|0x10000
     |          |          |          |
     |          |          |----------|   GDT
  512|----------|     ---> |Bootloader| -------> GDTR
     |Bl.data   |    /     |----------|0x7c00
     |Bl.code   | ---      |          |
  0x0·----------·          ·----------·0x0

Bootloader包含code和data，data中有GDT数据。
```

上电后，CPU从`0xFFFFFFF0`开始执行，跳转到到BIOS程序入口，然后将启动盘的第一个扇区作为引导代码(bootblock)加到内存`0x7c00`处，然后CPU在实模式下从`%cs=0 %ip=0x7c00`开始执行bootloader，所以链接时需要指定bootloader的入口（也可使用汇编可以在代码中直接指定）：

```text
-e start -Ttext 0x7C00
    # 指定入口函数为start，入口地址为0x7C00
    # 则bootloader的逻辑地址和加载地址均是相对0x7c00偏移的
    # 所以bootloader对于内存数据的访问是正常的
```

然后切换到32位保护模式（Enable A20），加载GDT（将GDT的首地址和长度加载到48位的GDTR寄存器中），此时1MB~4GB空间还没有代码，且bootloader的代码还要继续执行，所以设置`GDT.Base=0x00000000`，即`逻辑地址=线性地址=物理地址`，这样bootloader中的逻辑地址还是相对0x7c00偏移的。

- 加载内核

kernel保存在硬盘的第二个扇区，硬盘镜像通过dd命令实现：

```bash
dd if=/dev/zero of=$@ count=10000               # 建立10000扇区大小的硬盘镜像
dd if=$(bootblock_bin) of=$@ conv=notrunc       # 在第一个扇区放bootblock
dd if=$(kernel_bin) of=$@ seek=1 conv=notrunc   # 在第二个扇区开始放kernel
```

bootloader中调用bootmain，从汇编切换到C语言；在bootmain将kernel的elf头部加载到`0x10000`内存处：

```
    readseg((struct elfhdr*)0x10000, SECTSIZE * 8, 0);
```

但是`kernel.ld`中指定kernel的入口地址是`0xC0100000`，这样kernel中的逻辑地址都相对`0xC0100000`偏移的，现在kernel的elf头部加载到了物理地址`0x10000`处，导致`逻辑地址=线性地址=物理地址+0xC0100000`，所以为了加载kernel到内存`0x100000`（1MB）处时，需要处理一下实际访问的地址：

```
    // ph->p_va = 0xC0100000
    // 0xC0100000 & 0xFFFFFF = 0x100000
    readseg(ph->p_va & 0xFFFFFF, ph->p_memsz, ph->p_offset);
```

调用kernel入口函数，将CPU控制权交给kernel时，也要将处理`0xC0100000`的问题：

```
    ((void (*)(void))((struct elfhdr*)0x10000 & 0xFFFFFF))();
```

- 内核初始化

进入`kern_entry`后，仍然有`逻辑地址=线性地址=物理地址+0xC0100000`（kernel中的逻辑地址是相对于`0xC0100000`偏移的，而kernel本身被加载到了`0x100000`处，因为还没有使能分页，会直接将线性地址作为物理地址来访问），在kernel完成内存映射前，需要逻辑地址能正确该问到物理地址，需要做一点处理：

(1) 可以 将GDT.Base临时改成`-0xC0000000`，这样就有`逻辑地址-0xC0100000=线性地址=物理地址+0xC0100000-0xC0100000`，内存数据可以正常访问；
(2) 或者 设置`CR0.PG=1`使能分页，并建立一个临时页目录表`__boot_pgdir`加载到`CR3`，通过`__boot_pgdir`将线性地址`0xC0000000 ~ +4MB`映射到`0 ~ +4MB`；

上面2个方法，都可以临时保证kernel中的逻辑地址到物理地址正常的访问。


---
## MultiBoot

在kernel中加入multiboot支持，就可以直接使用grub引导kernel了。使用multiboot，实模式的GDT不需要自己设置，只需要设置保护模式下的GDT就好了。

---
## [UEFI](https://uefi.org/specifications)


---
# kernel

## driver/console

用于控制台基本输入/输出。

- 输出：向BIOS的CGA Buffer写入数据
- 输入：使用Serial接收来自KeyBoard的输入

## debug

debug中的`print_stackframe`可以打印函数调用栈。

- 相关寄存器

```
cs 代码的段寄存器（段选择子）
ss 栈的段寄存器（段选择子）
esp 栈的偏移地址寄存器
ebp 基址指针寄存器(默认的段地址为ss)

ss:esp 表示当前的栈顶地址
ss:(ebp + n) 表示从栈中访问数据
cs:eip 表示当前的代码执行地址
```

- 函数调用实例

caller调用func，在caller中会执行：

```
push arg2
push arg1
push ret
```

在func中会执行：

```
push ebp
mov  ebp, esp
push var1
push var2
```

函数调用栈是向低地址增长的：

```
| 栈底          | 高位地址
| ....          |
| arg2          |
| arg1          |
| ret addr      |
| caller中ebp值 | <- esp : esp指向ebp所在的地址 ('push ebp')
| var1          |          且func中的ebp=esp    ('mov ebp, esp')
| var2          |
| ....          | 低位地址

对于func来说：
  ss:(ebp + n)   ebp向上获取func的返回地址和参数
  ss:(ebp - n)   ebp向下得到func的局部变量
  ss:ebp         ebp本身保存着caller中的ebp值
                 使用ss:(ss:(ebp ± n))可以获取caller的返回地址、参数、局部变量等
```


## trap

`pic_init`用于初始化中断控制器，`idt_init`用于初始化中断描述符表（Interrupt Descriptor Table）

- IDT: 外部中断、软件中断和异常通过IDT处理

中断表初始化，一共可以设置0~255号中断。

- 外部中断：由处理器PIN脚接收外部硬件中断
- 软件中断：执行INT指令产生的中断，如`asm{INT 3}`即触发3号中断

中断调用过程：

```
         push(error_code, eip, cs, eflags)
INT {3} ----------------------------------->

    vector3() --->
                        push(trapframe)构建中断帧
        __alltraps(tf) --------------------------->
                                             根据中断号处理
            trap(tf) ---> trap_dispatch(tf) ----------------> switch(tf->tf_trapno) --->
                     从trap()返回
                ret -------------->
                       从stack中恢复trapframe
            __trapret ------------------------>

                iret ---> 返加到被INT打断的地方继续执行
```

## mm/pmm

- 分页表

获取到最大的物理内存地址`maxpa`后，将物理内存划分成4K的物理页，保存在`pages`中：

```
pages[maxpa / 4096]
```

对任意一个物理地址`pa`，`pages[pa/4096]`就是对应的物理页，pages由`pmm_manager`来管理物理页的alloc和free；
分页表用到的相关变量如下：

```
la         : 线性地址
va         : 虚拟地址
pa         : 物理地址
pages      : 所有空闲物理页的物理地址数组
ppn_t      : 一个物理页在pages中的下标
boot_pgdir : 保存页目录表PDT（Page Directory Entry数组）的物理地址，数组中共有2^10个pde_t，一个pde_t代表一个物理页（需要alloc）
pde_t      : 保存页表PT（Page Table Entry数组）的物理地址，数组中共有2^10个pte_t
pte_t      : 保存一个物理页的物理地址
```

创建分页表过程：

```
                           la : 用整型保存线性地址
boot_pgdir[PDX], pde_t, pte_t : 用整型保存物理地址(uintptr_t = uint32_t)
                           va : 用整型保存虚拟地址

                              boot_pgdir[PDX]          pde2page
                           -------------------> pde_t -----------> &pages[i]
     PDX                  /                     /  ^
la ·-------> LA[31:22] ---      KADDR(PDE_ADDR) |  |
   | PTX                                        |  | PADDR(PTE_ADDR)
   ·-------> LA[21:12]                          v  /
   | PPN                                         va
   ·-------> LA[31:12]                           /
   | PGOFF                               va[PTX] |
   ·-------> LA[11:0]                            |
                                                 v     pte2page
                                                pte_t -----------> &pages[j]
                                                 |
                                                 |     PTE_ADDR + PGOFF
                                                 ·----------------------> pa
```

虚拟地址和物理地址之间的转换关系：

```
      PPN
  ·--------> ppn_t <---------·
  |                          | page2pnn
  v                          v
            pa2page
 pa  --------------------> &pages[ppn_t]
    <--------------------
 ^ |        page2pa          ^ |
 | |                         | |
 | |  KADDR         kva2page | |
 | ·---------> va  ----------· |
 ·-----------     <------------·
      PADDR         page2kva
```

通过分页表，将`0xC0000000 ~ +0x38000000`的线性地址映射到了`0 ~ +0x38000000`的物理地址（ucore设置物理内存最大支持`0x38000000`Bytes）。
物理地址空间划分如下图：

```
  ·--------------------· 0xFFFFFFFF (4GB)
  |                    |
  |--------------------| 实际物理内存结束地址（比如计算机只有2GB内存）
  |                    |
  |       ~~~~~        | 空闲物理页，共n个
  |                    |
  |--------------------|
  |  n * sizeof(Page)  | pages指向的内存块，用于管理空闲物理页
  |--------------------|
  |                    |
  | (.text .data .bss) | pages, free_area, boot_pgdir, gdt[]等变量均在.data中
  |       Kernel       |
  |--------------------| 0x00100000 (1MB)
  |                    |
  |  Kernel ELF Header |
  |--------------------| 0x00010000
  |                    |
  |     Bootloader     |
  |--------------------| 0x00007c00
  |   Bootloader堆栈   |
  |                    |
  ·--------------------· 0x00000000
```

- 设置GDT

将GDT.Base设成`0x00000000`，则有`逻辑地址=线性地址=物理地址+KERNBASE`，因为使能了分页机制，所以会将线性地址映射到正确的物理地址再访问。

- first-fit分配算法

从`free_area`中alloc和free `n`个物理页的过程：

```
                                                     page
                                                        \
  ·----· <----> ·----· <----> ·----·       ·----· <-     ·----·     -> ·----·
  |    |        |    |        |    |  <=>  |    |   \    | n  |    /   |    |
  ·----·        |    |        ·----·       ·----·    \   |    |   /    ·----·
                |    |                                -> ·----· <-
                |    |                                   |    |
                ·----·                                   ·----·
```

## mm/vmm

建立分页表后，能访问的地址空间为`0xC0000000 ~ +0x38000000`，没有完全有到4GB的寻址空间。

32位CPU有4GB的寻址范围，实际中有些内存地址被硬件占用了或者物理内存不够4GB，这时线性地址到物理地址的映射，只用到了其中的一部分寻址空间。

为了使用全部的寻址空间，可以使用虚拟地址的概念：

```
va: 任意一个0~0xFFFFFFFF 地址空间的虚拟地址

va = 0xd0000000 : 通过分页表，可以正确访问物理地址
va = 0x00000010 : 没有对应的物理地址，产生缺页中断，需要建虚拟内存的映射
```

虚拟内存使用mm_struct和vma_struct来描述。
一个mm_struct以PDT为单位来管理虚拟内存，其中vma链表全部是属于PDT中的虚拟内存块。

```
  mm.mmap_list  va-block    mm.pgdir
        ^       ·-----·     ·-----·
        |       |     |     |     |
        v       |     |   ->|-----|
      vma1 ---> |-----|  /  |     |
        ^ \     |#####|--   |     |
        |  ·--> |-----|     |     |
        v       |     |     |     |
      vma2 ---> |-----|     |     |
        ^ \     |#####|--   |     |
        |  ·--> |-----|  \  |     |
        v       |     |   ->|-----|
       ...      ·-----·     ·-----·
```

## process

一个进程的基本描述：

```
struct mm_struct *mm   : 用于管理进程的内存数据
uintptr_t cr3          : 进程使用地页表
uintptr_t kstack       : 内核栈（用户进程进入内核态时用的，内核进程则作为进程运行）
struct context context : 进程上下文，进程调度时保存相关数据，用于进程恢复
struct trapframe *tf   : 中断帧，进程被中断时，保存相关数据，用于进程恢复
```

所有的进程使用`proc_list`链表保存，同时，使用`hash_list[]`来快速索引进程，使用pid作为hash值。

### 内核线程

- idleproc

第0个内核线程，简单理解为把当前kernel的执行状态构建成一个线程：

```
idleproc->mm      : 内核进程直接使用内核的内存管理，mm=NULL
idleproc->kstack  : bootstack，使用kernel的stack
idleproc->context : 使用kernel当前的上下文
idleproc->cr3     : boot_cr3，使用kernel的页表
```

- initproc

第1个内核线程，用于后续的初始化工作，以及用户进程的创建。

```
initproc->mm      : 内核进程直接使用内核的内存管理，mm=NULL
initproc->kstack  : 需要alloc
initproc->tf      : 从kstack的栈顶构建，tf->eip指向kernel_thread_entry，在kernel_thread_entry里面执行init_main
initproc->context : esp指向除去tf空间的栈顶，eip指向forkret（设置initproc当前执行到了forkret）
initproc->cr3     : boot_cr3，使用kernel的页表
```

### 用户进程

用户进程通过系统调用中断，执行`SYS_exec -> do_execve -> load_icode`，最后在将程序从文件中加载到内存中，退出中断时，通过`iret`指令，从用户进程序的trapframe恢复到用户进程序的运行。

(1) **userproc->mm**

创建mm_struct，管理用户进程内存数据，设置PDT（即mm->pgdir）；
设置用户虚拟内存空间，为elf文件各个段建立vma，插入到mm中；
分配物理内存空间给elf文件各个段，建立好`va->pa`的映射，将elf的各个段复制到相应的内存中；
设置用户栈，给栈分配虚拟内存空间和物理内存空间，并建立好`va->pa`的映射；

(2) **userproc->tf**

设置段选择子：`tf_cs, tf_ds, tf_es, tf_ss`；
设置mm准备的栈空间：`tf_esp`；
设置进程的入口地址：`tf_eip`

(3) **userproc->cr3**

使用mm->pgdir


### schedule

```
idleproc:
                                                         切换到initproc.context
    cpu_idle ---> schedule ---> proc_run ---> switch_to ------------------------>

initproc:
     ret(.context.eip)                           iret(.tf.eip)
    -------------------> forkret ---> __trapret ---------------> kernel_thread_entry --->

        init_main ---> schedule ---> ...
                  \
                   ·---> __alltraps ---> ...
```
