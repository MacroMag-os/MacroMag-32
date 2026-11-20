mkdir -p scripts

cat << 'EOF' > scripts/build_and_run.sh
#!/usr/bin/env bash
set -e

echo "[1/4] Building mm-init (VOS v2 Shell) in Rust..."
cd mm-init
RUSTFLAGS="-C linker=rust-lld -C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-musl
cd ..

echo "[2/4] Preparing initramfs root filesystem..."
mkdir -p build/initramfs/{bin,dev,proc,sys,etc,mnt} build/iso/boot/grub
cp mm-init/target/x86_64-unknown-linux-musl/release/mm-init build/initramfs/init
chmod +x build/initramfs/init
cp boot/grub/grub.cfg build/iso/boot/grub/

if [ ! -f build/initramfs/bin/busybox ]; then
    curl -L -o build/initramfs/bin/busybox https://busybox.net
    chmod +x build/initramfs/bin/busybox
fi

if [ ! -f build/iso/boot/vmlinuz ]; then
    curl -L -o build/iso/boot/vmlinuz https://alpinelinux.org
fi

echo "[3/4] Packing initramfs.img and virtual disk..."
(cd build/initramfs && find . -print0 | cpio --null -ov --format=newc | gzip -9 > ../iso/boot/initramfs.img)
if [ ! -f build/disk.img ]; then
    dd if=/dev/zero of=build/disk.img bs=1M count=128 status=none
fi

echo "[4/4] Booting MacroMag OS in QEMU..."
qemu-system-x86_64 \
    -kernel build/iso/boot/vmlinuz \
    -initrd build/iso/boot/initramfs.img \
    -hda build/disk.img \
    -append "console=ttyS0 quiet loglevel=3 rdinit=/init" \
    -m 512M \
    -nographic
EOF
chmod +x scripts/build_and_run.sh

cat << 'EOF' > CLA.md
# MacroMag OS Contributor License Agreement (CLA)

Thank you for your interest in contributing to **MacroMag OS** ("the Project"), led by Adrian Morawski ("Project Lead").

By submitting a Pull Request, patch, or other contribution to this repository, you agree to the following terms:

1. **Copyright Ownership:** You retain full copyright ownership of your original contributions.
2. **Grant of License to Project Lead:** You grant the Project Lead a perpetual, worldwide, non-exclusive, royalty-free, irrevocable license to use, reproduce, modify, distribute, and sublicense your contributions under both:
   - The **GNU General Public License v3.0 (GPLv3)** for Community/Open Source editions, and
   - **Commercial / Proprietary licenses** for MacroMag Enterprise editions and add-ons.
3. **Original Work:** You represent that your contribution is your original creation and that you have the legal right to grant the above license.
EOF

printf "\n/build/\n" >> .gitignore
git add .gitignore scripts/build_and_run.sh CLA.md
git commit -m "chore: add automated QEMU build script and Contributor License Agreement (CLA)"
git push origin main
