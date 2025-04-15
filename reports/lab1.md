# Lab1 Report

## 实现功能

- `TaskControlBlock`结构体添加`task_syscall_times`字段
- `TaskManager`方法中实现指定系统调用的自增和次数查询
- `syscall/mod.rs`中增加syscall次数
- `syscall/process`中实现trace系统调用
 
## 简答

1. 

SBI版本: rust-sbi 0.3.0-alpha2 RISC-V SBI 1.0.0

- ch2b_bad_address: 试图在0x0位置写入数据，这个地址是非法地址，报错PageFault in application, bad addr = 0x0
- ch2b_bad_instructions: 试图在user mode使用sret指令，从而造成非法，报错IllegalInstruction in application, kernel killed it
- ch2b_bad_register: 试图在user mode访问sstatus寄存器，从而造成非法，报错IllegalInstruction in application, kernel killed it

2.

1. 刚进入__restore时，sp指向内核栈的栈顶;执行完trap之后恢复上下文、切换应用程序时返回user mode

2. 恢复sstatus、sepc、sscratch三个csr。其中sscratch指向用户栈的栈顶;sepc记录Trap发生之前执行的最后一条指令的地址，返回user mode可以继续执行程序;sstatus控制supervisor mode中cpu的执行状态

3. x2：在__alltraps中保存sp，需要基于它来找到每个寄存器应该被保存到的正确的位置，因此没有保存，在__restore中自然无需恢复;x4：tp寄存器，一般情况下不会用到，无需进行保存和恢复

4. sp指向用户栈，sscratch指向内核栈

5. sret指令;sret从内核态切换回trap之前的位置

6. sp指向内核栈，sscratch指向用户栈

7. ecall


## 荣誉准则

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。