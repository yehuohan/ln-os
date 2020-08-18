
# rcore(x86_64)

基于Rust开发的x86_64 OS。


# VGA Display

对`0xb8000`地址进行读写。


# CPU Exceptions

## The interrupt calling convention

- 传递前6个整型参数的寄存器：rdi, rsi, rdx, rcx, r8, r9
- 其余的参数：stack
- 返回结果：rax, rdx

## Preserved and scratch registers

| Preserved (callee-saved)          | Scratch (caller-saved)               |
| -                                 | -                                    |
| rbp, rbx, rsp, r12, r13, r14, r15 | rax, rcx, rdx, rsi, rdi, r8, r9, r10 |

- callee: 被调用的函数
- caller: 调用callee的函数


## The interrupt stack frame

- 正常函数调用 

正常函数调用使用`call`指令，返回使用`ret`指令。
