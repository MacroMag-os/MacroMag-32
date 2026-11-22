#!/usr/bin/env bash
set -e

cargo build --release --target x86_64-unknown-linux-musl

ROOTFS="build_rootfs"
ISO_DIR="build_iso"
rm -rf "$ROOTFS" "$ISO_DIR"
mkdir -p "$ROOTFS" "$ISO_DIR/boot/grub"

wget -q "https://dl-cdn.alpinelinux.org/alpine/v3.20/releases/x86_64/alpine-minirootfs-3.20.3-x86_64.tar.gz" -O /tmp/alpine.tar.gz
tar -xzf /tmp/alpine.tar.gz -C "$ROOTFS"
rm -f /tmp/alpine.tar.gz

mkdir -p "$ROOTFS/lib/firmware"
git clone --depth 1 https://git.kernel.org/pub/scm/linux/kernel/git/firmware/linux-firmware.git /tmp/fw
cp -r /tmp/fw/iwlwifi* /tmp/fw/rtl_nic /tmp/fw/rtlwifi /tmp/fw/ath* "$ROOTFS/lib/firmware/" 2>/dev/null || true
rm -rf /tmp/fw

cp target/x86_64-unknown-linux-musl/release/vos "$ROOTFS/init"
cp target/x86_64-unknown-linux-musl/release/mur "$ROOTFS/usr/bin/mur"
chmod +x "$ROOTFS/init" "$ROOTFS/usr/bin/mur"
mkdir -p "$ROOTFS/etc/vos" "$ROOTFS/var/lib/mur"

cp kernel/vmlinuz "$ISO_DIR/boot/vmlinuz"
(cd "$ROOTFS" && find . -print0 | cpio --null -ov --format=newc 2>/dev/null | gzip -9 > "../$ISO_DIR/boot/initramfs.img")

cat << 'EOF' > "$ISO_DIR/boot/grub/grub.cfg"
set default=0
set timeout=3
menuentry "MacroMag OS (Full Server Edition x86_64)" {
    linux /boot/vmlinuz console=tty0 console=ttyS0 rw
    initrd /boot/initramfs.img
}
EOF

grub-mkrescue -o MacroMag-x86_64.iso "$ISO_DIR"
zip -9 MacroMag-x86_64-ISO.zip MacroMag-x86_64.iso
