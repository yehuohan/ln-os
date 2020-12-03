
## qemu

- 运行参数

```
-s
# 等同于参数 -gdb tcp::1234

-p port
# 改变gdb连接端口号

-S
# 启动后暂停CPU，需要在monitor中输入'c'，才能让qemu继续模拟工作

-serial dev
-parallel dev
-monitor dev
# 重定向虚拟串口／虚拟并口／monitor到Host的设备dev（如stdio, file, pipe等）
```

- 调试命令

将qemu的monitor重定向到stdio，则可以在终端里输入命令进行调试。
 
```
help    # 查看qemu帮助
info    # 查询qemu支持的相关信息
x       # 显示虚拟地址的数据
xp      # 显示物理地址的数据
p|print     # 计算并显示表达式的值
r|registers # 显示所有寄存器的内容
```

## gdb

```
target remote localhost:1234
# 连接到qemu调试
```

## lldb

```
gdb-remote 1234
# 连接到qemu调试
```
