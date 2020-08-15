
# Linux

- [linux](https://git.kernel.org/)
- [KernelNewbies](https://kernelnewbies.org/)
- [Documention](https://www.kernel.org/doc/html/latest/)
- [linux-insides](https://github.com/0xAX/linux-insides/blob/master/SUMMARY.md)

---
# startup(x86_64)

## Boot to kernel

Bootloader的入口在`linux/arch/x86/boot/header.S`中的`_start`，链接设置为`linux/arch/x86/boot/setup.ld`：

```
_start -> start_of_setup -> [init segment-registers, stack, bss] -> main()
```

在`linux/arch/x86/boot/main.c`的`main()`函数中，会进行Boot参数设置、Console初始化、CPU检测、保护模式切换等操作，之后从32位保护模式继续切换到64位模式：

```
main() -> go_to_protected_mode() -> protected_mode_jump -> .Lin_pm32 -> startup_32 -> startup_64
```

在`linux/arch/x86/boot/compressed/head_64.S`的`startup_64`函数中进行GDT初始化、解压缩`extract_kernel`等操作，然后跳转到kernel入口`start_kernel`：

```
startup_64 ->
    [linux/arch/x86/kernel/head_64.S]startup_64 -> [linux/arch/x86/kernel/head64.c]__startup_64() -> x86_64_start_kernel() ->
        [linux/init/main.c]start_kernel()
```
