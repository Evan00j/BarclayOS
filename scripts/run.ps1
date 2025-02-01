# QEMU file path
set QEMU 'C:\Program Files\qemu\qemu-system-riscv64.exe'

cargo build

# Start QEMU
& $QEMU -machine virt -bios default -nographic -serial mon:stdio --no-reboot -kernel target/riscv64gc-unknown-none-elf/debug/barclay_kernel
