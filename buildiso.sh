#!/usr/bin/env bash
set -e

echo "=== [1/7] Kompilacja Rust workspace (VOS + MUR) ==="
cargo build --release --target x86_64-unknown-linux-musl

ROOTFS="build_rootfs"
ISO_DIR="build_iso"
rm -rf "$ROOTFS" "$ISO_DIR"
mkdir -p "$ROOTFS" "$ISO_DIR/boot/grub"

echo "=== [2/7] Pobieranie bazy musl + dynamicznego linkera (Alpine minirootfs) ==="
ALPINE_REL="https://dl-cdn.alpinelinux.org/alpine/latest-stable/releases/x86_64"
ROOTFS_TAR=$(wget -qO- "$ALPINE_REL/" | grep -oE 'alpine-minirootfs-[0-9]+\.[0-9]+\.[0-9]+-x86_64\.tar\.gz' | head -n 1)
echo "Pobieranie: $ROOTFS_TAR"
wget -q "$ALPINE_REL/$ROOTFS_TAR" -O /tmp/alpine.tar.gz
tar -xzf /tmp/alpine.tar.gz -C "$ROOTFS"
rm -f /tmp/alpine.tar.gz

echo "=== [3/7] Instalacja statycznego curl (HTTPS) i certyfikatow SSL dla MUR ==="
mkdir -p "$ROOTFS/etc/ssl/certs" "$ROOTFS/usr/bin"
wget -q "https://curl.se/ca/cacert.pem" -O "$ROOTFS/etc/ssl/certs/ca-certificates.crt"
wget -q "https://github.com/moparisthebest/static-curl/releases/latest/download/curl-amd64" -O "$ROOTFS/usr/bin/curl"
chmod +x "$ROOTFS/usr/bin/curl"

echo "=== [4/7] Wypakowanie niezbednych modulow jadra Linux LTS (modloop-lts) ==="
mkdir -p "$ROOTFS/lib"
unsquashfs -f -d /tmp/modloop kernel/modloop-lts
rm -rf /tmp/modloop/modules/*/kernel/drivers/gpu \
       /tmp/modloop/modules/*/kernel/drivers/media \
       /tmp/modloop/modules/*/kernel/drivers/infiniband \
       /tmp/modloop/modules/*/kernel/sound
cp -r /tmp/modloop/modules "$ROOTFS/lib/"
rm -rf /tmp/modloop

echo "=== [5/7] Pobieranie lekkiego firmware sieciowego z Alpine ==="
ALPINE_MAIN="https://dl-cdn.alpinelinux.org/alpine/latest-stable/main/x86_64"
for pkg in linux-firmware-rtl_nic linux-firmware-rtlwifi linux-firmware-iwlwifi; do
    PKG_FILE=$(wget -qO- "$ALPINE_MAIN/" | grep -oE "${pkg}-[0-9][^\"]*\.apk" | head -n 1)
    if [ -n "$PKG_FILE" ]; then
        wget -q "$ALPINE_MAIN/$PKG_FILE" -O "/tmp/$PKG_FILE"
        tar -xzf "/tmp/$PKG_FILE" -C "$ROOTFS" --exclude=".PKGINFO" --exclude=".SIGN*" 2>/dev/null || true
        rm -f "/tmp/$PKG_FILE"
    fi
done

echo "=== [6/7] Instalacja binarek MacroMag VOS i MUR ==="
cp target/x86_64-unknown-linux-musl/release/vos "$ROOTFS/init"
cp target/x86_64-unknown-linux-musl/release/mur "$ROOTFS/usr/bin/mur"
chmod +x "$ROOTFS/init" "$ROOTFS/usr/bin/mur"
mkdir -p "$ROOTFS/etc/vos" "$ROOTFS/var/lib/mur"

echo "=== [7/7] Pakowanie initramfs i generowanie hybrydowego ISO (BIOS + UEFI) ==="
cp kernel/vmlinuz "$ISO_DIR/boot/vmlinuz"
(cd "$ROOTFS" && find . -print0 | cpio --null -ov --format=newc 2>/dev/null | gzip -9 > "../$ISO_DIR/boot/initramfs.img")

cat << 'GRUBEOF' > "$ISO_DIR/boot/grub/grub.cfg"
set default=0
set timeout=3
menuentry "MacroMag OS (Full Server Edition x86_64)" {
    linux /boot/vmlinuz console=tty0 console=ttyS0 rw
    initrd /boot/initramfs.img
}
GRUBEOF

grub-mkrescue -o MacroMag-x86_64.iso "$ISO_DIR"
