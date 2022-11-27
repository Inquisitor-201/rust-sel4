.extern _bss
.extern _bss_end

.section .text.start
.global _start
_start:
    # a0 = hartid, a1 = dtb
    /* save the parameters passed */
    mv s0, a0
    mv s2, a1
    la sp, (elfloader_stack_alloc + (1 << 12))
    jal clear_bss

    mv a0, s0
    mv a1, s2
    jal main

.section .text
.global clear_bss

clear_bss:
    la a5, _bss
    la a6, _bss_end
1:  sb x0, 0(a5)
    addi a5, a5, 1
    blt a5, a6, 1b
    ret

.section .bss.stack
.global elfloader_stack_alloc
elfloader_stack_alloc:
    .space 1 << 12

