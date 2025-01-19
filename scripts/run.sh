#!/bin/bash
set -xue

# QEMU file path
QEMU=qemu-system-riscv64

cargo build

# Start QEMU
$QEMU -machine virt -bios default -nographic -serial mon:stdio --no-reboot -kernel target/riscv64gc-unknown-none-elf/debug/barclay_kernel
