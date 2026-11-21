
#!/usr/bin/env bash
set -e

echo "=== Building MacroMag OS x86_64 Live/Install ISO ==="

WORK_DIR="$(pwd)/build_work"
ROOTFS="$WORK_DIR/rootfs"
ISO_DIR="$WORK_DIR/iso"
TARGET="x86_64-unknown-linux-musl"

rm -rf "$WORK_DIR"
mkdir -p "$ROOTFS"/{bin,sbin,etc,proc,sys,dev,tmp,root,mnt,usr/bin,usr/sbin,var/cache/mur,boot/grub}
mkdir -p "$ISO_DIR/boot/grub"

# 1. Kompilacja narzędzi MacroMag w Rust (statycznie pod musl z głównego Workspace)
echo "[1/5] Compiling Rust workspace (vos, mur)..."
rustup target add "$TARGET" 2>/dev/null || true
cargo build --release --target "$TARGET"

# VOS jako główny proces startowy (/init - PID 1) oraz jako /bin/vos
cp "target/$TARGET/release/vos" "$ROOTFS/init"
cp "target/$TARGET/release/vos" "$ROOTFS/bin/vos"

# Klient menedżera pakietów MUR w /usr/bin/mur
cp "target/$TARGET/release/mur" "$ROOTFS/usr/bin/mur"

chmod +x "$ROOTFS/init" "$ROOTFS/bin/vos" "$ROOTFS/usr/bin/mur"

# 2. Pobranie statycznego BusyBox + Fastfetch + Dropbear SSH
echo "[2/5] Downloading static system utilities (BusyBox, Fastfetch)..."
BUSYBOX_URL="https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox"
curl -L -f "$BUSYBOX_URL" -o "$ROOTFS/bin/busybox"
chmod +x "$ROOTFS/bin/busybox"

for cmd in sh mount umount ls cp mv rm cat echo ip udhcpc reboot mkdir dmesg; do
    ln -sf /bin/busybox "$ROOTFS/bin/$cmd"
done

# Pobranie statycznego Fastfetch (Linux x86_64)
FASTFETCH_URL="https://github.com/fastfetch-cli/fastfetch/releases/latest/download/fastfetch-linux-amd64.tar.gz"
curl -L -f "$FASTFETCH_URL" -o "$WORK_DIR/fastfetch.tar.gz"
tar -xzf "$WORK_DIR/fastfetch.tar.gz" -C "$WORK_DIR"
cp "$WORK_DIR/fastfetch-linux-amd64/usr/bin/fastfetch" "$ROOTFS/usr/bin/fastfetch" || true
chmod +x "$ROOTFS/usr/bin/fastfetch" || true

# Kopiowanie Dropbear SSH z systemu budującego (jeśli zainstalowany)
if command -v dropbear >/dev/null; then
    cp "$(which dropbear)" "$ROOTFS/sbin/dropbear"
fi

# 3. Konfiguracja użytkowników i logo fastfetch
echo "[3/5] Configuring users and branding..."
echo "root:macromag" > "$ROOTFS/etc/macromag_users"
echo "root:x:0:0:root:/root:/bin/vos" > "$ROOTFS/etc/passwd"

cat << 'EOF' > "$ROOTFS/etc/fastfetch_logo.txt"
 __  __   __  __ 
|  \/  | |  \/  |  MacroMag OS v0.1
| |\/| | | |\/| |  Zero-Telemetry Linux
| |  | | | |  | |  Shell: VOS (Rust)
|_|  |_| |_|  |_|  Arch: x86_64
EOF

# 4. Pakowanie rootfs do initramfs.img
echo "[4/5] Packing initramfs..."
(cd "$ROOTFS" && find . -print0 | cpio --null -ov --format=newc | gzip -9 > "$ISO_DIR/boot/initramfs.img")

# 5. Kopiowanie jądra Linuxa i budowa hybrydowego ISO przez GRUB
echo "[5/5] Building bootable MacroMag-x86_64.iso..."
if [ -f "kernel/arch/x86/boot/bzImage" ]; then
    cp "kernel/arch/x86/boot/bzImage" "$ISO_DIR/boot/vmlinuz"
elif [ -f "kernel/bzImage" ]; then
    cp "kernel/bzImage" "$ISO_DIR/boot/vmlinuz"
else
    sudo cp $(ls /boot/vmlinuz-* | head -n 1) "$ISO_DIR/boot/vmlinuz" && sudo chmod 644 "$ISO_DIR/boot/vmlinuz"
fi

cat << 'EOF' > "$ISO_DIR/boot/grub/grub.cfg"
set timeout=2
set default=0

menuentry "MacroMag OS v0.1 (VOS Shell + SSH)" {
    linux /boot/vmlinuz console=ttyS0 console=tty0 quiet
    initrd /boot/initramfs.img
}
EOF

grub-mkrescue -o MacroMag-x86_64.iso "$ISO_DIR"
echo "=== DONE! Created MacroMag-x86_64.iso ==="



