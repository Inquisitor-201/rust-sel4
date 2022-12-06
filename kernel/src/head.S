    .section .boot.text, "ax"
    .globl _start
_start:
    # la  t1, boot_stack
    # add     t0, a0, 1
    # slli    t0, t0, 15 # hart_id * stacksize
    # add  sp, t1, t0
    la sp, boot_stack_top
    jal init_kernel

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 1 << 15
    .globl boot_stack_top
boot_stack_top:
