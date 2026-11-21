rustup target add riscv64gc-unknown-none-elf
cargo build --release --target riscv64gc-unknown-none-elf
qemu-system-riscv64 -machine virt -bios none -nographic -kernel target/riscv64gc-unknown-none-elf/release/macromag-kernel
