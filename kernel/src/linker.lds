OUTPUT_ARCH(riscv)
ENTRY(_start)
KERNEL_ELF_BASE = 0x80400000;

SECTIONS
{
    . = KERNEL_ELF_BASE;

    .boot : {
        *(.boot.text)
        *(.boot.rodata)
        *(.boot.data)
        . = ALIGN(4K);
    }

    ki_boot_end = .;

    .text : {
        . = ALIGN(4K);
        *(.text)
    }

    . = ALIGN(4K);
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    sbss_with_stack = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}