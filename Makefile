arch ?= x86_64
target ?= $(arch)-rust_os
rust_os := target/$(target)/debug/librust_os.a
kernel := build/kernel-$(arch).bin
iso := build/img-$(arch).iso

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.nasm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.nasm, \
	build/arch/$(arch)/%.o, $(assembly_source_files))

.PHONY: all clean run iso kernel
RUST_TARGET_PATH = ./
all: $(kernel)

clean: 
	@rm -r build

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -m 0.5G

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

kernel:
	 
	@RUST_TARGET_PATH=$(dir $(abspath $(lastword $(MAKEFILE_LIST)))) xargo build --target $(target)

build/arch/$(arch)/%.o: src/arch/$(arch)/%.nasm
	@mkdir -p $(shell dirname $@)
	@nasm -f elf64 $< -o $@