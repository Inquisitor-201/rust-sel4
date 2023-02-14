# rust-sel4

使用rust语言复现的sel4项目。

## 运行项目

```shell
cd tools/elfloader
make run
```

----------------------------------------------------------------

## 目前进度
1. 支持riscv64指令集
2. boot阶段完成（包括cap_table的初始化、untyped caps的分配）
3. 编写了用户态程序，可以向内核发出syscall

## 未来计划
1. 实现ipc、thread的相关系统调用
2. 完善文档

