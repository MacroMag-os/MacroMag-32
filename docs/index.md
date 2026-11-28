# MacroMag OS

**MacroMag OS** is an experimental, lightweight, GNU-free Linux distribution written from scratch in **Rust** and **C**. Created by Polish developer **Adrian Morawski** (*zaldyrek* / *maldyrek*), the system is specifically engineered to combat e-waste by giving older and low-spec 32-bit/64-bit hardware a fast, modern second life.

## Core Philosophy & Architecture

Unlike traditional Linux distributions that rely on heavy userland stacks, systemd, and GNU coreutils, MacroMag OS strips the operating system down to its bare essentials:

* **Zero GNU Dependencies:** Built with a completely independent, minimal userland written in Rust and C.
* **VOS as PID 1:** Instead of traditional init systems like systemd or OpenRC, MacroMag OS boots directly into **VOS** (*Virtualized Object Shell*), an object-oriented shell acting as the primary init process.
* **MUR Package Manager:** Software installation and system updates are handled by **MUR** (*MacroMag User Repository*), a fast and lightweight custom package manager.
* **E-Waste Prevention:** Designed with minimal RAM and CPU overhead in mind, making it ideal for reviving legacy laptops and older x86/32-bit machines.
* **Author** Adrian Morawski

## Quick Links

* **Official Source Code:** [GitHub Repository (MacroMag-os/MacroMag-32)](https://github.com/Macromag-os/Macromag-32)
* **License:** GNU General Public License v3.0 (GPLv3)
* **Author:** Adrian Morawski

## Getting Started

1. Download the latest bootable ISO image from the [GitHub Releases](https://github.com/Macromag-os/Macromag-32/releases) page.
2. Boot the image in a virtual machine (QEMU, VirtualBox) or write it to a USB drive for bare-metal testing on legacy hardware.
3. Explore the **VOS** environment and manage packages using **MUR**.
