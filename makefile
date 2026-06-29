.PHONY: all build link iso run clean

all: run

build:
	CARGO_INCREMENTAL=0 cargo build -Zjson-target-spec

boot.o: src/arch/i686/boot.asm
	nasm -f elf32 src/arch/i686/boot.asm -o boot.o

link: build boot.o
	ld -m elf_i386 -n -T linker.ld -o kernel.bin boot.o target/i686-unknown-none/debug/libpikoKernel.a

iso: link
	mkdir -p isodir/boot/grub
	cp kernel.bin isodir/boot/kernel.bin
	cp boot/grub/grub.cfg isodir/boot/grub/grub.cfg
	grub2-mkrescue -o pikoOS.iso isodir

run: iso
	qemu-system-i386 -cdrom pikoOS.iso

clean:
	cargo clean
	rm -f boot.o kernel.bin pikoOS.iso
	rm -rf isodir