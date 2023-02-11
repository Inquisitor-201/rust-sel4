.section .text
.global _sel4_start

_sel4_start:
    la sp, __stack_top
    jal sel4_start_root
    mv t0, a0

.section .bss
__stack_base:
    .align 16
    .space (1 << 16)
__stack_top: