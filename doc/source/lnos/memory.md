
x86架构从硬件上支持分段（Segmentation）和分页（Paging）两种内存管理机制。

# Segmentation

- 实模式：使用20位地址总线，寻址空间为1Mb，使用段寄存器Seg(CS,DS,SS等)和偏移Ofs寻址

实模式下使用8086中的段偏移寻址：

```
逻辑地址Ofs : 程序中看到的地址为逻辑地址，如'int* p = &val'中的指针p
段地址Seg   : 即段寄存器中保存的值
物理地址PA  : Seg << 4 + Ofs
```

- 保护模式：使用段寄存器Seg(CS,DS,SS等，也叫段选择子)、偏移Ofs和GDT寻址

段寄存器保存访问GDT的下标，GDT中包含物理内存的地址和访问权限；

Segmentation使用了Virtual Memory的技术，虚拟内存地址经过“翻译”后，得到物理地址：

![virtual memory](img/segmentation-virtual.svg)

> Virtaul Address通过CS和GDT计算


# Paging

Virtual地址将一块块的地址空间分成Pages；Physial地址将一块块的地址空间分成Frames；
每一个Page可以映到一个Frame：

![Pages and Frames](img/pages-frames.svg)


## Page Tables

x86_64使用4级分页表，每级页表通过64位地址的下标索引：

![4-Level Paging](img/4-level-paging.svg)

每一级页表（Page Table）下标占9Bits（2^9=512），可以索引512个页表项（Page Table Entry）；
每一个Entry占8Bytes，故一个页表需要4KiB空间保存（8*512=4KiB）；
最低12Bits是Page内的偏移地址，所以一个Page为4KiB（刚好一个Page可以保存一个页表）。

从Virtual地址到Physial地址的转化示例：

![4 Level Paging Translation](img/4-level-paging-translation.svg)

CR3保存L4页表的物理地址；
L4保存L3页表的物理地址；
L3保存L2页表的物理地址；
L2保存L1页表的物理地址；
具体是页表中的哪个页表项Entry，则由Index索引。

```
Virtual Address = 0x803FE7F5CE

000000000 000000000 000000001 000000000 111111111 001111111 010111001100
------------------- ========= --------- ========= --------- ============
   Sign Extension    Level-4   Level-3   Level-2    Level=1 Offset=0x5CE
                     Index=1   Index=0  Index=511 Index=127

附：2^3=8，使用8进制表示Virtual Address，可以表示 'SSSSSS_AAA_BBB_CCC_DDD_EEEE' 这种格式。
```

可以计算，共有：

- 1 个 L4
- 512 个 L3
- 512 x 512 个 L2
- 512 x 512 x 512 个 L1

## Page Talbe Entry

每一个Entry占8Bytes，格式如下：

```
| Bit(s) | Name                  | Meaning                                                                                      |
| ---    | ---                   | ---                                                                                          |
| 0      | present               | the page is currently in memory                                                              |
| 1      | writable              | it's allowed to write to this page                                                           |
| 2      | user accessible       | if not set, only kernel mode code can access this page                                       |
| 3      | write through caching | writes go directly to memory                                                                 |
| 4      | disable cache         | no cache is used for this page                                                               |
| 5      | accessed              | the CPU sets this bit when this page is used                                                 |
| 6      | dirty                 | the CPU sets this bit when a write to this page occurs                                       |
| 7      | huge page/null        | must be 0 in P1 and P4, creates a 1GiB page in P3, creates a 2MiB page in P2                 |
| 8      | global                | page isn't flushed from caches on address space switch (PGE bit of CR4 register must be set) |
| 9-11   | available             | can be used freely by the OS                                                                 |
| 12-51  | physical address      | the page aligned 52bit physical address of the frame or the next page table                  |
| 52-62  | available             | can be used freely by the OS                                                                 |
| 63     | no execute            | forbid executing code on this page (the NXE bit in the EFER register must be set)            |
```


分页后，各级页表指向的都是4K对齐的Page，即0~11Bit均为0，所以Entry只用12~51来保存物理地址就足够了。

## Translation Lookaside Buffer

4-Level分页表，每一次内存访问（Translation），需要4次内存读取操作。
x86提供了TLB来缓存最近Translation的地址，如果该问的是TLB中的地址，可以跳过Translation过程。


# Accessing Page Tables

当CPU开启分页机制后，Kernel中访问的地址就全部是Virtual地址，
所以若要修改Page Table中的Entry，就需要访问保存Page Table的Physical地址，
在Kernel中就需要知道映到到Physical的Virtual地址。

> lnos中使用bootloader在引导kernel前，就已经做好分页表了，并且把0xb800的virtual地址，映射到了0xb800的物理地址，所以直接访问0xb800才实现了对VGA的访问。

所以，为了访问Page Table，就是将一些虚拟地址，映射到Page Table的物理地址。

## Identify Mapping

最粗暴的方法，即将所有Page Table的物理地址映射成一样的虚拟地址。

![Identify Mapping](img/identity-mapped-page-tables.svg)

## Map at a Fixed Offset

将Page Table的虚拟地址，放在一个seperate的地址空间中。

![Map at a Fixed Offset](img/page-tables-mapped-at-offset.svg)

## Map the Complete Physical Memory

将所有的物理地址映射到相同 ***大小*** 的虚拟地址空间。

![Map the Complete Physical Memory](img/map-complete-physical-memory.svg)

## Temporary Mapping

对于物理内存较小的设备，可以只映射1个L4，其余L3、L2、L1当需要时才临时映射。

## Recursive Page Tables

将一个L4其中的一个Entry映射成当前L4的物理地址。

![Recursive Page Tables](img/recursive-page-table.png)

图中，L4将第511个Entry，映射成了L4的物理地址。


```
AAA = Level-4 Index
BBB = Level-3 Index
CCC = Level-2 Index
DDD = Level-1 Index
EEEE = Offset
RRR = L4中映射L4物理地址的Entry的Index（图中为511）

将不同的Index设置成RRR，则可以访问不同Level页表的物理地址：

SSSSSS_AAA_BBB_CCC_DDD_EEEE : Page
SSSSSS_RRR_BBB_CCC_DDD_EEEE : Level-1
SSSSSS_RRR_BBB_CCC_DDD_EEEE : Level-2
SSSSSS_RRR_RRR_RRR_DDD_EEEE : Level-3，例如 RRR = 511，DDD = 2，EEEE=8，则是访问 &L3[2] + 8 （L3第2个Entry的物理地址+8）
SSSSSS_RRR_RRR_RRR_RRR_EEEE : Level-3，例如 RRR = 511，EEEE=8，则是访问 &L4 + 8 （L4的物理地址+8）
```

> bootloader使能features='map_physical_memory'时，则是使用Map the Complete Physical Memory
> bootloader使能features='recursive_page_table'时，则是使用Recursive Page Tables
