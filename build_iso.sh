#!/bin/bash
set -e

echo "=== 0. Instalacja narzędzi budujących ==="
sudo apt-get update
sudo apt-get install -y xorriso cpio curl git build-essential kmod

echo "=== 1. Kompilacja statycznej binarki mm-init (z wbudowanym MUR) ==="
rustup target add x86_64-unknown-linux-musl
cd mm-init
cargo build --release --target x86_64-unknown-linux-musl
cd ..

echo "=== 2. Tworzenie struktury prawdziwego systemu (rootfs) ==="
rm -rf build_tmp
mkdir -p build_tmp/rootfs/{bin,sbin,etc,proc,sys,dev,usr/bin,var/lib/mur,lib/modules}

# Główny proces init w Rust (zawiera wbudowany klient MUR)
cp mm-init/target/x86_64-unknown-linux-musl/release/mm-init build_tmp/rootfs/init
chmod +x build_tmp/rootfs/init

# Identyfikacja systemu
if [ -f etc/os-release ]; then
    cp etc/os-release build_tmp/rootfs/etc/os-release
fi

# Statyczny BusyBox (zapewnia mount, mdev, modprobe, udhcpc, sh, ls, cat itp.)
curl -L https://busybox.net -o build_tmp/rootfs/bin/busybox
chmod +x build_tmp/rootfs/bin/busybox
for cmd in sh mount umount mdev modprobe insmod udhcpc ip ifconfig ping ls cat mkdir rm cp mv clear; do
    ln -s busybox build_tmp/rootfs/bin/$cmd
done

# Skrypt konfigurujący sieć DHCP dla udhcpc
mkdir -p build_tmp/rootfs/usr/share/udhcpc
cat << 'EOF' > build_tmp/rootfs/usr/share/udhcpc/default.script
#!/bin/sh
[ "$1" = "bound" ] || [ "$1" = "renew" ] && {
    ip addr add "$ip/$mask" dev "$interface"
    [ -n "$router" ] && ip route add default via "$router" dev "$interface"
    echo "nameserver 1.1.1.1" > /etc/resolv.conf
    echo "nameserver 8.8.8.8" >> /etc/resolv.conf
}
EOF
chmod +x build_tmp/rootfs/usr/share/udhcpc/default.script

# Kopiowanie sterowników sprzętowych (USB, dyski, karty sieciowe) dla prawdziwego PC
KVER=$(uname -r)
sudo cp -r /lib/modules/$KVER build_tmp/rootfs/lib/modules/
sudo depmod -b build_tmp/rootfs $KVER

# Pakowanie initramfs
cd build_tmp/rootfs
find . | cpio -o -H newc | gzip > ../initramfs.img
cd ../..

echo "=== 3. Jądro Linux oraz bootloader Limine (UEFI + BIOS) ==="
sudo cp /boot/vmlinuz-$KVER build_tmp/vmlinuz
sudo chmod 644 build_tmp/vmlinuz

git clone https://github.com --branch=v8.x-binary --depth=1 build_tmp/limine
make -C build_tmp/limine

echo "=== 4. Generowanie hybrydowego obrazu MacroMag-32.iso ==="
mkdir -p build_tmp/iso_root/boot/limine
mkdir -p build_tmp/iso_root/EFI/BOOT

cp build_tmp/vmlinuz build_tmp/iso_root/boot/
cp build_tmp/initramfs.img build_tmp/iso_root/boot/

cat << 'EOF' > build_tmp/iso_root/boot/limine/limine.conf
timeout: 3

/MacroMag OS (Native Bare-Metal)
    protocol: linux
    kernel_path: boot():/boot/vmlinuz
    module_path: boot():/boot/initramfs.img
    cmdline: quiet loglevel=3 console=tty0
EOF

cp build_tmp/limine/limine-bios.sys build_tmp/limine/limine-bios-cd.bin build_tmp/limine/limine-uefi-cd.bin build_tmp/iso_root/boot/limine/
cp build_tmp/limine/BOOTX64.EFI build_tmp/iso_root/EFI/BOOT/

xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --efi-boot boot/limine/limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image --protective-msdos-label \
    build_tmp/iso_root -o MacroMag-32.iso

./build_tmp/limine/limine bios-install MacroMag-32.iso

echo "=== GOTOWE! MacroMag-32.iso jest gotowy do nagrania na pendrive ==="
