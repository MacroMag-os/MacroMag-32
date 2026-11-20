#!/bin/bash
set -e

echo "=== 0. Instalacja zależności ==="
sudo apt-get update
sudo apt-get install -y xorriso cpio curl git build-essential

echo "=== 1. Kompilacja statycznej binarki mm-init (Rust musl) ==="
rustup target add x86_64-unknown-linux-musl
cd mm-init
cargo build --release --target x86_64-unknown-linux-musl
cd ..

echo "=== 2. Tworzenie struktury systemu plików (initramfs) ==="
rm -rf build_tmp
mkdir -p build_tmp/rootfs/{bin,etc,proc,sys,dev}

# Kopiowanie naszego procesu VOS jako główny /init
cp mm-init/target/x86_64-unknown-linux-musl/release/mm-init build_tmp/rootfs/init
chmod +x build_tmp/rootfs/init

# Kopiowanie /etc/os-release do obrazu systemu
if [ -f etc/os-release ]; then
    cp etc/os-release build_tmp/rootfs/etc/os-release
fi

# Poprawny link do statycznej binarki busybox (x86_64)
curl -L https://busybox.net -o build_tmp/rootfs/bin/busybox
chmod +x build_tmp/rootfs/bin/busybox
ln -s busybox build_tmp/rootfs/bin/mount

# Pakowanie initramfs.img
cd build_tmp/rootfs
find . | cpio -o -H newc | gzip > ../initramfs.img
cd ../..

echo "=== 3. Pobieranie jądra Linux oraz bootloadera Limine ==="
sudo cp /boot/vmlinuz-$(uname -r) build_tmp/vmlinuz || sudo cp /boot/vmlinuz* build_tmp/vmlinuz
sudo chmod 644 build_tmp/vmlinuz

git clone https://github.com --branch=v8.x-binary --depth=1 build_tmp/limine
make -C build_tmp/limine

echo "=== 4. Składanie obrazu ISO ==="
mkdir -p build_tmp/iso_root/boot/limine
mkdir -p build_tmp/iso_root/EFI/BOOT

cp build_tmp/vmlinuz build_tmp/iso_root/boot/
cp build_tmp/initramfs.img build_tmp/iso_root/boot/

cat << 'EOF' > build_tmp/iso_root/boot/limine/limine.conf
timeout: 2

/MacroMag OS (Prototype v0.1 - Text Mode)
    protocol: linux
    kernel_path: boot():/boot/vmlinuz
    module_path: boot():/boot/initramfs.img
    cmdline: quiet loglevel=0 console=tty0 console=ttyS0
EOF

cp build_tmp/limine/limine-bios.sys build_tmp/limine/limine-bios-cd.bin build_tmp/limine/limine-uefi-cd.bin build_tmp/iso_root/boot/limine/
cp build_tmp/limine/BOOTX64.EFI build_tmp/iso_root/EFI/BOOT/

xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --efi-boot boot/limine/limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image --protective-msdos-label \
    build_tmp/iso_root -o MacroMag-32.iso

./build_tmp/limine/limine bios-install MacroMag-32.iso

echo "=== SUKCES! Wygenerowano MacroMag-32.iso ==="
