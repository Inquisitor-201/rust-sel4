# Building
TARGET := riscv64gc-unknown-none-elf
MODE ?= release
IMAGE_ELF := target/$(TARGET)/$(MODE)/elfloader
IMAGE_BIN := $(IMAGE_ELF).bin
KERNEL_DIR := ../../kernel
KERNEL_ELF := ${KERNEL_DIR}/target/$(TARGET)/$(MODE)/kernel
ARCHIVE_BIN := archive.archive.o.cpio
SDCARD := /dev/sdb
CPUS ?= 1
	
ifeq ($(MODE), release)
	BUILD_MODE := --release
else
	BUILD_MODE := 
endif

BOARD ?= qemu
# 默认使用opensbi-1.0
SBI ?= opensbi
ifeq ($(SBI), rustsbi)
	BOOTLOADER := ../../bootloader/$(SBI)-$(BOARD).bin
else 
	BOOTLOADER := ../../bootloader/fw_jump.bin
endif

fu740_BOOTLOADER_SIZE := 1310720

KERNEL_ENTRY_PA := 0x80200000

# Run fu740
fu740-SERIALPORT	= /dev/ttyUSB0
fu740-BURNER	= ../tools/kflash.py

# Binutils
OBJDUMP := riscv64-linux-gnu-objdump
OBJCOPY := rust-objcopy --binary-architecture=riscv64

image: ${ARCHIVE_BIN}
	@echo Platform: $(BOARD)
	@cargo build ${BUILD_MODE}

${ARCHIVE_BIN}: ${KERNEL_ELF}
	@mkdir -p target && cp ${KERNEL_ELF} target/ && cd target && bash -c "echo kernel" | cpio --create > ${ARCHIVE_BIN} && rm kernel

${KERNEL_ELF}:
	@make -C ${KERNEL_DIR}

readelf:
	@riscv64-linux-gnu-readelf -a $(IMAGE_ELF) > image-readelf.txt

asm:
	@$(OBJDUMP) -lS $(IMAGE_ELF) > image.asm

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
# cargo install cargo-binutils --vers =0.3.3
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(IMAGE_BIN): image
	@$(OBJCOPY) $(IMAGE_ELF) --strip-all -O binary $@

clean:
	@cargo clean

run: $(IMAGE_BIN) asm
	qemu-system-riscv64 \
		-machine virt \
		-m 1G	\
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(IMAGE_BIN),addr=$(KERNEL_ENTRY_PA) \
		-smp $(CPUS)
        # -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

gdb:
	qemu-system-riscv64 \
		-m 1G	\
		-machine virt -nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(IMAGE_BIN),addr=$(KERNEL_ENTRY_PA) \
		-smp $(CPUS) \
		-s -S
# -drive file=$(FS_IMG),if=none,format=raw,id=x0 \
# -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \

monitor:
	riscv64-unknown-elf-gdb -ex 'file $(IMAGE_ELF)' \
		-ex 'set arch riscv:rv64' \
		-ex 'target remote localhost:1234'

# -ex 'add-symbol-file ../user/libc-tests/entry-static.exe'\

.PHONY: env image gdb monitor clean run
