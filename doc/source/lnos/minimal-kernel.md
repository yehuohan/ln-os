
# Minimal Kernel

一个最简内核，第一步是要编译一个与Host系统、rust-std无关的二进制程序（但是可以使用alloc、core等库，以及不依赖std的其它库）；其次实现内核的引导启动。

这里使用[bootimage](https://github.com/rust-osdev/bootimage)实现x86_64平台的引导。

最简内核的编译还需要以下设置：

- .cargo/config.toml: 配置cargo编译参数
- target_arch: 使用json定义OS平台参数


## VAG

使用VGA实现简单的屏幕（80x25）输出，本质上就是对内存地址0xb800进行读写。


## Test

有了基本输出，就可以实现基本kernel的测试框架。

