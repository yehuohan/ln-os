
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
控制寄存器CR3 : CR3[31:12]为保存页目录表（Page Directory Entry数组）的物理地址
页目录表PDE   : PDE[31:12]为保存页表（Page Table Entry数组）的物理址
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

## mm/pmm

- 建立分页表

获取到最大的物理内存地址`maxpa`后，将物理内存划分成4K的物理页，保存在`pages`中：

```
pages[maxpa / 4096]
```

对任意一个物理地址`pa`，`pages[pa/4096]`就是对应的物理页，pages由`pmm_manager`来管理物理页的alloc和free；
使用`boot_pgdir`保存页目录表（Page Directory Entry数组）的物理地址，同时使用alloc物理页来保存页表（Page Table Entry数组）的物理地址，PTE保存的则是`pa/4096`。

```
boot_pgdir映射：
                      线性地址 -> 物理地址
KERNBASE ~ KERNBASE + KMEMSIZE -> 0 ~ KMEMSIZE
```

通过分页表，虽然kernel本身加载到了`0 ~ 1GB`的物理内存空间，但kernel中使用的是`3GB ~ 4GB`的逻辑地址。

物理地址空间划分如下图：

```
  ·--------------------· 0xFFFFFFFF (4GB)
  |                    |
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

- 建立GDT

将GDT.Base设成`0x00000000`，则有`逻辑地址=线性地址=物理地址+KERNBASE`，因为使能了分页机制，所以会将线性地址映射到正确的物理地址再访问。

## trap

- IDT: 外部中断、软件中断和异常通过IDT处理

中断表初始化，一共可以设置0~255号中断。

- 外部中断：由处理器PIN脚接收外部硬件中断
- 软件中断：执行INT指令产生的中断，如`asm{INT 3}`即触发3号中断

```
中断调用：
INT {3} -> vector3() -> __alltraps(tf) -> trap(tf) -> trap_dispatch(tf) -> switch(tf->tf_trapno) -> ...
```
