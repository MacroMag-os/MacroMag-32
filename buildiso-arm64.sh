#!/usr/bin/env bash
set -e

echo "=== [1/7] Kompilacja Rust workspace (VOS + MUR) dla ARM64 ==="
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
cargo build --release --target aarch64-unknown-linux-musl

ROOTFS="build_rootfs_arm64"
ISO_DIR="build_iso_arm64"
rm -rf "$ROOTFS" "$ISO_DIR"
mkdir -p "$ROOTFS" "$ISO_DIR/boot/grub" "$ISO_DIR/EFI/BOOT"

echo "=== [2/7] Pobieranie bazy musl + dynamicznego linkera (Alpine minirootfs aarch64) ==="
ALPINE_REL="https://alpinelinux.org"
ROOTFS_TAR=$(wget -qO- "$ALPINE_REL/" | grep -oE 'alpine-minirootfs-[0-9]+\.[0-9]+\.[0-9]+-aarch64\.tar\.gz' | head -n 1)
echo "Pobieranie: $ROOTFS_TAR"
wget -q "$ALPINE_REL/$ROOTFS_TAR" -O /tmp/alpine-arm64.tar.gz
tar -xzf /tmp/alpine-arm64.tar.gz -C "$ROOTFS"
rm -f /tmp/alpine-arm64.tar.gz

echo "=== [3/7] Instalacja statycznego curl (HTTPS) i certyfikatow SSL dla MUR ==="
mkdir -p "$ROOTFS/etc/ssl/certs" "$ROOTFS/usr/bin"
wget -q "https://curl.se/ca/cacert.pem" -O "$ROOTFS/etc/ssl/certs/ca-certificates.crt"
wget -q "https://github.com" -O "$ROOTFS/usr/bin/curl"
chmod +x "$ROOTFS/usr/bin/curl"

echo "=== [4/7] Pobieranie jadra Linux LTS i modulow (modloop-lts aarch64) ==="
NETBOOT_URL="$ALPINE_REL/netboot"
wget -q "$NETBOOT_URL/vmlinuz-lts" -O "$ISO_DIR/boot/vmlinuz"
wget -q "$NETBOOT_URL/modloop-lts" -O /tmp/modloop-lts-arm64

mkdir -p "$ROOTFS/lib"
unsquashfs -f -d /tmp/modloop_arm64 /tmp/modloop-lts-arm64
rm -rf /tmp/modloop_arm64/modules/*/kernel/drivers/gpu \
       /tmp/modloop_arm64/modules/*/kernel/drivers/media \
       /tmp/modloop_arm64/modules/*/kernel/drivers/infiniband \
       /tmp/modloop_arm64/modules/*/kernel/sound
cp -r /tmp/modloop_arm64/modules "$ROOTFS/lib/"
rm -rf /tmp/modloop_arm64 /tmp/modloop-lts-arm64

echo "=== [5/7] Pobieranie lekkiego firmware sieciowego z Alpine (aarch64) ==="
ALPINE_MAIN="https://alpinelinux.org"
for pkg in linux-firmware-rtl_nic linux-firmware-rtlwifi linux-firmware-iwlwifi; do
    PKG_FILE=$(wget -qO- "$ALPINE_MAIN/" | grep -oE "${pkg}-[0-9][^\"]*\.apk" | head -n 1)
    if [ -n "$PKG_FILE" ]; then
        wget -q "$ALPINE_MAIN/$PKG_FILE" -O "/tmp/$PKG_FILE"
        tar -xzf "/tmp/$PKG_FILE" -C "$ROOTFS" --exclude=".PKGINFO" --exclude=".SIGN*" 2>/dev/null || true
        rm -f "/tmp/$PKG_FILE"
    fi
done

echo "=== [6/7] Instalacja binarek MacroMag VOS i MUR (aarch64) ==="
cp target/aarch64-unknown-linux-musl/release/vos "$ROOTFS/init"
cp target/aarch64-unknown-linux-musl/release/mur "$ROOTFS/usr/bin/mur"
chmod +x "$ROOTFS/init" "$ROOTFS/usr/bin/mur"
mkdir -p "$ROOTFS/etc/vos" "$ROOTFS/var/lib/mur"

echo "=== [7/7] Pakowanie initramfs i generowanie bootowalnego ISO UEFI (ARM64) ==="
(cd "$ROOTFS" && find . -print0 | cpio --null -ov --format=newc 2>/dev/null | gzip -9 > "../$ISO_DIR/boot/initramfs.img")

cat << 'GRUBEOF' > "$ISO_DIR/boot/grub/grub.cfg"
set default=0
set timeout=3
menuentry "MacroMag OS (Full Server Edition ARM64 UEFI)" {
    linux /boot/vmlinuz console=tty0 console=ttyAMA0,115200 rw
    initrd /boot/initramfs.img
}
GRUBEOF

cp "$ISO_DIR/boot/grub/grub.cfg" "$ISO_DIR/EFI/BOOT/grub.cfg"

# Generowanie natywnego bootloadera UEFI dla ARM64 (BOOTAA64.EFI)
grub-mkimage -O arm64-efi -o "$ISO_DIR/EFI/BOOT/BOOTAA64.EFI" -p /boot/grub \
    part_gpt part_msdos fat iso9660 normal boot linux configfile search search_fs_file search_label

# Tworzenie partycji rozruchowej FAT32 (efiboot.img) dla urzadzen bare-metal i VM
dd if=/dev/zero of=/tmp/efiboot-arm64.img bs=1M count=16
mkfs.vfat /tmp/efiboot-arm64.img
mmd -i /tmp/efiboot-arm64.img ::/EFI ::/EFI/BOOT
mcopy -i /tmp/efiboot-arm64.img "$ISO_DIR/EFI/BOOT/BOOTAA64.EFI" ::/EFI/BOOT/
mcopy -i /tmp/efiboot-arm64.img "$ISO_DIR/EFI/BOOT/grub.cfg" ::/EFI/BOOT/
cp /tmp/efiboot-arm64.img "$ISO_DIR/boot/efiboot.img"
rm -f /tmp/efiboot-arm64.img

# Skladanie hybrydowego ISO bootowalnego z plyty i pendrive'a na czystym sprzecie ARM64
xorriso -as mkisofs \
    -o MacroMag-aarch64.iso \
    -volid "MACROMAG_ARM64" \
    -e boot/efiboot.img \
    -no-emul-boot \
    -isohybrid-gpt-basdat \
    "$ISO_DIR"

echo "=== GOTOWE: MacroMag-aarch64.iso ==="
