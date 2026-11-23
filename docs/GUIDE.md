# Running & Installing MacroMag OS in VMware

Although **MacroMag OS** is designed and engineered for real 64-bit (`x86_64`) bare-metal hardware, **VMware Workstation Player** or **VMware Workstation Pro** provides a convenient, isolated environment to test the system, explore the **Virtualized Object Shell (VOS)**, and verify bare-metal disk installation (`sys.install`) without risking your primary drive.

---

## 1. Prerequisites

- **Hypervisor:** VMware Workstation Player (free for personal use) or VMware Workstation Pro (Windows / Linux).
- **ISO Image:** Download the latest `MacroMag-x86_64.iso` from the **[GitHub Releases](https://github.com/MacroMag-os/MacroMag-32/releases)** page, or compile it directly from source:

```bash
git clone https://github.com/MacroMag-os/MacroMag-32.git
cd MacroMag-32
chmod +x buildiso.sh
./buildiso.sh
```

---

## 2. Virtual Machine Configuration

Follow these exact settings to ensure full compatibility with the current MacroMag OS kernel and storage drivers:

1. Launch VMware and select **Create a New Virtual Machine**.
2. Choose **Typical (Recommended)** and click **Next**.
3. Select **Installer disc image file (iso)**, browse to your `MacroMag-x86_64.iso` file, and click **Next**.
4. Under **Guest Operating System**, choose **Linux** and set the Version to **Other Linux 6.x kernel 64-bit** (or *Other Linux 5.x kernel 64-bit*).
5. Name your virtual machine (e.g., `MacroMag-OS`) and set the disk capacity to **2 GB – 4 GB**. Choose **Store virtual disk as a single file** for better I/O consistency.
6. Before powering on, click **Customize Hardware...** and verify the following:
   - **Memory (RAM):** Minimum **512 MB** (1024 MB recommended).
   - **Network Adapter:** Set to **NAT** (allows `udhcpc` to automatically assign an IP address and enables `mur sync` over HTTPS).
   - **Firmware Type (Options -> Advanced):** Ensure firmware is set to **Legacy BIOS** (default). *Note: UEFI/GPT support is tracked under Issue #31.*
   - **Hard Disk Controller:** Verify the virtual disk uses **SATA** or **IDE** rather than NVMe/SCSI so the kernel maps it cleanly to `/dev/sda`.

---

## 3. Booting & Exploring VOS

Power on the virtual machine. The stripped-down Linux kernel will boot directly into `/init` (**Virtualized Object Shell**), bypassing traditional SysV/systemd overhead.

Once at the VOS prompt, test the built-in strongly-typed object modules and POSIX compatibility:

```text
# Inspect system resources and CPU details
sys.cpu()
sys.mem()
sys.uptime()

# List detected storage devices and network interfaces
disk.list()
net.interfaces()

# Standard POSIX pipes and commands also work seamlessly
ps aux | grep syslogd
```

---

## 4. Testing Disk Installation (`sys.install`)

To test installing MacroMag OS from the virtual CD-ROM (`/dev/sr0`) onto the VM's virtual hard drive:

1. Confirm the target disk device name (usually `/dev/sda`) using:
   ```text
   disk.list()
   ```
2. Run the native VOS installer:
   ```text
   sys.install("/dev/sda")
   ```
3. Once `sys.install` finishes copying `vmlinuz`, `initramfs.img`, root directories, and configuring GRUB, reboot the virtual machine:
   ```text
   sys.reboot()
   ```
4. **Disconnect the ISO:** During reboot (or while powered off), open **VM Settings -> CD/DVD (SATA)** and uncheck **Connect at power on** so the VM boots directly from `/dev/sda`.

---

## 5. Testing the MUR Package Manager

With the VM network adapter set to **NAT**, `udhcpc` configures networking and DNS (`1.1.1.1` / `8.8.8.8`) automatically at boot. You can sync and install packages from the **[MacroMag User Repository (MUR)](https://github.com/MacroMag-os/mur)**:

```text
mur sync
mur list
mur info <package-name>
mur install <package-name>
```

---

## 6. Troubleshooting

| Issue | Cause | Solution |
| :--- | :--- | :--- |
| **`disk.list()` does not show `/dev/sda`** | VMware defaulted to a SCSI/NVMe controller unsupported by the minimal kernel config. | Power off the VM, remove the virtual hard disk, and re-add a new disk selecting **SATA** or **IDE** as the disk type. |
| **VM fails to boot (`Operating System not found`)** | Firmware is set to UEFI instead of Legacy BIOS. | Open **VM Settings -> Options -> Advanced** and switch Firmware type to **BIOS**. |
| **`mur sync` fails to connect** | Network adapter is disconnected or set to Host-Only. | Switch the VM Network Adapter to **NAT** and verify connectivity with `net.interfaces()` or `ping 1.1.1.1`. |
| **VM boots back into Live ISO after `sys.install()`** | Virtual CD/DVD drive still has higher boot priority than the hard drive. | Disconnect the ISO image in **VM Settings -> CD/DVD** or press `Esc` at the VMware splash screen to select the Hard Drive. |
