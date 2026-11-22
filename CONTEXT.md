# MacroMag OS — Linux Distribution System Creating Context
*Snapshot of architecture, completed milestones, and remaining roadmap for people continuity.*

### Code: Rust/C/C++

## 1. System Architecture Overview
- **Project:** MacroMag OS (`MacroMag-os/MacroMag-32`) — modular, zero-telemetry Linux distribution (`x86_64`) designed for bare-metal PCs, servers, and legacy hardware (anti-e-waste).
- **Boot & Build (`buildiso.sh`):** Statically compiles Rust workspace (`x86_64-unknown-linux-musl`), bundles static BusyBox (`ash`, `fdisk`, `mkfs.ext2`, `tar`, `sha256sum`, `syslogd`, etc.), static `curl` (HTTPS), `fastfetch`, `dropbear` SSH, and `grub-install`, producing a hybrid bootable `MacroMag-x86_64.iso`.
- **Init & Virtualised Object Shell (`userspace/vos`):**
  - Runs directly as `/init` (`PID 1`).
  - Mounts `/dev` (`devtmpfs`), `/dev/pts` (`devpts`), `/proc`, `/sys`, `/tmp`.
  - Reaps orphaned zombie processes (`waitpid(-1, &mut status, WNOHANG)`).
  - Launches `syslogd`, `klogd`, configures DNS (`1.1.1.1`, `8.8.8.8`), and brings up network interfaces via `ip` + `udhcpc`.
  - Dual-mode parser: executes VOS strongly-typed object pipelines (when `(` precedes `|`) and delegates standard POSIX pipes/redirections (`|`, `>`, `<`, `&`) to `/bin/sh -c`.
  - Built-in object modules: `sys.ps()`, `sys.mem()`, `sys.uptime()`, `sys.cpu()`, `sys.install("/dev/sdX")`, `sys.reboot()`, `sys.poweroff()`, `disk.list()`, `net.interfaces()`, `fs.ls()`, `fs.cat()`, `mur.sync()`, `mur.list()`, `mur.install()`.
- **Package Manager (`userspace/mur`):**
  - Connects to `https://github.com` (`packages/*/mur.pkg`).
  - Syncs repository archive (`main.tar.gz`) via `curl` + `tar` (zero `git` dependency).
  - Parses `name`, `version`, `description`, `url`, `bin`, and verifies `sha256` checksums before installing binaries to `/usr/bin`.
  - Commands: `mur sync`, `mur list`, `mur info <pkg>`, `mur install <pkg>`, `mur remove <pkg>`.

## 2. Completed Milestones (Closed Issues)
- **[CORE] `PID 1` Stability:** Implemented zombie process reaping (`waitpid`), `/dev/pts` pseudo-terminal mounting, and `syslogd`/`klogd` background logging.
- **[SHELL] Dual-Mode Pipeline Execution:** Separated VOS typed object pipelines from standard Linux POSIX pipes/redirections.
- **[INSTALLER] Bare-Metal Rootfs Copy (`sys.install`):** Updated `sys_install` to mount boot media (`/dev/sr0`, `/dev/sdb1`) to copy `vmlinuz` and `initramfs.img`, copy `/bin`, `/sbin`, `/usr`, `/etc`, dynamically configure `root=/dev/...` in `grub.cfg`, and unmount cleanly.
- **[MUR] Git-Free Sync & Manifests:** Replaced `git` with `curl` + `tar`, added full `mur.pkg` parsing (`version`, `bin`, `description`, `sha256`), and added `mur info`.
- **[VOS] Hardware Object Modules:** Implemented `disk.list()`, `net.interfaces()`, and `sys.cpu()` in `vos_builtins.rs`.
- **[CI/CD] Automated ISO Build:** Verified passing build workflow (`Build and Release MacroMag ISO`).

## 3. Remaining Open Issues (Next Steps)
1. **`#29` [INIT] Service Supervisor:** Build a Rust `ServiceManager` reading `/etc/vos/services.toml` to supervise and auto-restart background daemons (`udhcpc`, `dropbear`, `syslogd`) + `sys.services()` command.
2. **`#31` [INSTALLER] GPT & UEFI Support:** Detect `/sys/firmware/efi` in `sys.install()` to support GPT + 256MB FAT32 ESP + `grub-install --target=x86_64-efi` alongside Legacy BIOS MBR.
3. **`#32` [CORE] Persistent Root & `/etc/fstab`:** Parse `/etc/fstab` on boot, run `fsck`, and switch/mount persistent disk storage in `rw` mode so changes survive `sys.reboot()`.
4. **`#33` [INSTALLER] Swap & `zram` Support:** Enable compressed `zram` RAM at boot and create `/swapfile` during `sys.install()` for 512 MB–2 GB legacy hardware.
5. **`#34` [SECURITY] Multi-User Login (`userspace/login`):** Integrate password authentication against `/etc/macromag_users` at boot and add `user.add()` / `user.passwd()` built-ins.
6. **`#37` [MUR] Dependency Resolution & Updates:** Support `depends=pkg1,pkg2` in `mur.pkg`, track installed versions in `/var/lib/mur/installed.db`, and implement `mur upgrade`.
7. **`#38` [SYSTEM] `musl` Dynamic Linker & Shared Libraries:** Bundle `ld-musl-x86_64.so.1`, `libssl`, `libcrypto`, and `zlib` to run standard dynamically linked Linux binaries.
8. **`#39` [NETWORK] Wi-Fi & Firewall:** Add `wpa_supplicant` + wireless firmware, `net.wifi()` VOS commands, static IP configuration, and `nftables`/`iptables` rules.
9. **`#40` [SHELL] Line Editing, History & Tab Completion:** Replace raw `stdin().read_line()` with `rustyline` or `termios` raw mode for Up/Down arrow history (`~/.vos_history`) and `Tab` completion.
10. **`#41` [KERNEL] Custom Linux Kernel `.config`:** Add optimized `defconfig` in `kernel/` with built-in `NVMe`, `AHCI`, `USB`, `ext4`, `vfat`, and NIC/Wi-Fi drivers.
11. **`#42` [RELEASE] Automated Release Artifacts:** Ensure tagged releases automatically attach verified `MacroMag-x86_64.iso` images.

## 4. Key Repository Files
- `Cargo.toml` — Workspace root (`userspace/vos`, `userspace/mur`)
- `buildiso.sh` — Live/Install ISO builder script
- `userspace/vos/src/main.rs` — `PID 1` init, environment setup, and VOS/POSIX REPL loop
- `userspace/vos/src/vos_builtins.rs` — Built-in VOS modules (`sys`, `fs`, `disk`, `net`, `mur`) and `sys_install()`
- `userspace/vos/src/vos_parser.rs` & `vos_executor.rs` & `vos_core.rs` — Object pipeline lexer, parser, and execution engine
- `userspace/mur/src/main.rs` — MUR package manager CLI
- `userspace/login/` — Authentication module (ready for Issue #34 integration)

## 5. Our goal is Build Real Linux Based System powerful Like Ubuntu Text Version
- `no qemu or emulators only real 64-bit Architecture`
- `vos and mur connect in to Macromag-os Linux System`
- `in future we thinking about GUI and enterprise function add`
