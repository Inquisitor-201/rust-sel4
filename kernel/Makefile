# Building
TARGET := riscv64gc-unknown-none-elf
MODE ?= release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
SDCARD := /dev/sdb
CPUS ?= 2
	
ifeq ($(MODE), release)
	BUILD_MODE := --release
else
	BUILD_MODE := 
endif


BOARD ?= qemu
# 默认使用opensbi-1.0
SBI ?= opensbi
ifeq ($(SBI), rustsbi)
	BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin
else 
	BOOTLOADER := ../bootloader/fw_jump.bin
endif

fu740_BOOTLOADER_SIZE := 1310720

# KERNEL ENTRY 0x80200000
ifeq ($(BOARD), qemu)
	KERNEL_ENTRY_PA := 0x80200000
else ifeq ($(BOARD), fu740)
	KERNEL_ENTRY_PA := 0x80200000
endif

# Run fu740
fu740-SERIALPORT	= /dev/ttyUSB0
fu740-BURNER	= ../tools/kflash.py

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# 反汇编并把文件保存到target/riscv64gc-unknown-none-elf/release/os.S
asm:
	@$(OBJDUMP) -all $(KERNEL_ELF) > kernel.asm

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
# cargo install cargo-binutils --vers =0.3.3
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

user-apps:
	@make -C ../user elf

kernel: user-apps
	@echo Platform: $(BOARD)
	@cp src/linker-$(BOARD).ld src/linker.ld
	@cargo build --offline $(BUILD_MODE) --features "board_$(BOARD)"
	@rm src/linker.ld

clean:
	@cargo clean

run: build
	@qemu-system-riscv64 \
		-machine virt \
		-m 1G	\
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
        -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
		-smp $(CPUS)

gdb:
	@qemu-system-riscv64 \
		-m 1G	\
		-machine virt -nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
		-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
		-smp $(CPUS) \
		-s -S

monitor:
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' \
		-ex 'set arch riscv:rv64' \
		-ex 'target remote localhost:1234'

# -ex 'add-symbol-file ../user/libc-tests/entry-static.exe'\

.PHONY: build env kernel clean run user-apps fs-img handin
