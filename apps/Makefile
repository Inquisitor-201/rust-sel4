all: build
	riscv64-linux-gnu-objdump -lS target/riscv64gc-unknown-none-elf/release/rootserver > rootserver.asm

build:
	cargo build --release

.PHONY: build all