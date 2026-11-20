 # MacroMag OS: Modular Online Computing Architecture by Adrian Morawski

<p align="center">
  <img src="banner.gif" alt="MacroMag OS Demo" width="100%">
</p>


 [![DOI](https://zenodo.org/badge/1421486557.svg)](https://doi.org/10.5281/zenodo.24201693)

[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/16688/badge)](https://www.bestpractices.dev/projects/16688)

[![License: GPL v3](https://shields.io)](https://gnu.org)
[![Status: Experimental](https://shields.io)]()

> *"Those who are crazy enough to think they can change the world, are the ones who do."* — Steve Jobs

Modern computing is trapped in a cycle of planned obsolescence. Mainstream operating systems and locked-down hardware (*eFUSEs*, remote kill-switches, mandatory cloud telemetry) actively turn perfectly functional electronics into e-waste.

**MacroMag is an experimental hardware-software architecture designed for digital sovereignty, modularity, and zero-telemetry online execution.**

Instead of patching forty-year-old legacy standards, we are building a clean, minimalist ecosystem focused on object-oriented command execution, hardware-level sandboxing, and modular hardware design.

## 🚀 Quick Start

### Prerequisites
To build and boot the MacroMag linux based system, install the Rust RISC-V target and QEMU:
- **Rust toolchain** (`rustup target add riscv64gc-unknown-none-elf`)
- **QEMU** (`qemu-system-riscv64`)

### Build & Run on QEMU
1. Clone the repository and navigate to the kernel directory:
   ```bash
   git clone https://github.com/MacroMag-os/MacroMag-32
   cd MacroMag-32/kernel
   ```
2. Make a linux based system graphic interfacce:
   ```bash
   cargo build --release --target riscv64gc-unknown-none-elf
   ```
3. Give us a iso file (headless serial console):
   ```bash
   qemu-system-riscv64 -machine virt -bios none -nographic -kernel target/riscv64gc-unknown-none-elf/release/macromag-kernel
   ```
4. Complete the VOS Shell (Virtualized object shell)
5. Still Update The exsiting os.
   
### Pre-built Prototype (ISO & Binaries)
Don't want to compile from source? Download pre-built prototype images (`MacroMag-32.iso`) and test utilities directly from our **[GitHub Releases](https://github.com/MacroMag-os/MacroMag-32)** page.

## 2. Pure Linux Kernel Core (`x86_64` & `RISC-V`)

Instead of reinventing hardware drivers from scratch, MacroMag leverages a stripped-down, deterministic **Linux Kernel** purely as a hardware abstraction layer.

* **Zero Telemetry, Full Hardware Support:** No background cloud polling, no legacy GNU userland clutter, and immediate compatibility with standard PCs, legacy laptops (Anti E-Waste), and open **RISC-V** boards.
* **VOS Native Execution:** The kernel boots directly into our **Rust/C** runtime and the Virtualized Object Shell (**VOS**), bypassing traditional Bash/POSIX bottlenecks while keeping strict process sandboxing.

### Enterprise Cloud Architecture (Opt-In): While the Community edition operates strictly air-gapped with zero telemetry, MacroMag Enterprise introduces an optional, deterministic cloud control plane. Leveraging VOS typed object pipelines over encrypted channels, organizations can orchestrate fleets of x86_64 and RISC-V nodes, deploy atomic updates ("Online But Always Updating"), and manage hardware modules remotely—available both as a managed cloud service and a self-hosted on-premise solution.

## I want to create a Linux distro, but not a classic one... I want to create something that doesn't exist yet: a free Open version (on which the Enterprise version will also be based), and Enterprise is a paid add-on for Open and my way of making money (but so as not to fall into Red Hat's clutches, this Enterprise system will be cheaper, so I will sell it at a bargain price to smaller IT companies so they buy it). The kernel (Linux) is available, so I don't have to create it, but I want to build such a deep interface that it looks 'not like Linux' and has the style of Apple macOS  but done my way, with my famous logo MM > MacroMag. Now, I know it's damn hard, but complicating a non-existent kernel on my own would be impossible, so I'll use the Linux

## ⚡ Core Architectural Pillars

### 1. The Virtualized Object Shell (VOS)
MacroMag replaces traditional raw-text shells with an object-oriented execution environment.
* **Typed Memory Pipelines:** Commands exchange strongly-typed objects directly in memory rather than parsing fragile text streams (`grep`/`awk`/`sed` hacks).
* **Native Sandbox Syntax:** Process isolation is built directly into the shell syntax. Every untrusted execution runs inside a lightweight microkernel sandbox by default.

### 2. Graphic iNTERFACE
* **Zero Telemetry, 100% Determinism:** No background cloud polling, no hidden corporate firmware. CPU cycles go strictly into user-defined tasks.
* **Open Architecture First:** Rather than reinventing proprietary silicon, MacroMag targets open **64-bit RISC-V** hardware and FPGA implementations, leveraging existing modern toolchains (Rust/C/LLVM).
* **Self-Contained Core:** Designed to operate completely air-gapped with a lean, auditable set of essential system tools.

### 3. Long-Term R&D: Magnetic Modularity & Phantom BIOS
Our long-term hardware vision aims to eliminate single-point-of-failure motherboards and fight e-waste:
* **Decentralized Phantom BIOS:** Each physical hardware module carries its own minimal firmware descriptor, registering its capabilities dynamically upon connection.
* **Hot-Swappable Hardware Nodes:** Researching fault-tolerant, magnetic/pogo-pin interconnects for deterministic real-time module replacement without system crashes.

## 🤝 Project Leadership & Team

* **Adrian Morawski (@adrianmorawski713)** — **Founder & CEO** (Project Creator, Lead Architect & Vision)
* **Alan Morawski (@sxalanek)** — **Core Administrator** (Code, Legal Compliance & Security)
* **Maldyrek (@maldyrek)** — **Core Administrator** (Operations & Community Management)

## 💜 Support
**We Need Your Support To: - Create Official Website (macromag.org) of project and paid for code evolution progress mode!**
# if you want give us a donate, Go To opencollective Site **[Support](https://opencollective.com/macromag).** Page

---

## 💻 Paradigm Shift: VOS Syntax Concept

**Traditional CLI (Raw String Parsing):**
```bash
vm-list | grep "RUNNING" | awk '{if (\$3 > 512) print \$1}'
```

**MacroMag VOS (Typed Object Pipeline):**
```powershell
vm.list() | filter status == "running" && memory > 512MB | select name
```

**Sandboxed Execution:**
```powershell
spawn --isolated --image:web_server -> Core.VM
```

---

## 🛠️ Current Status & Roadmap

We are currently in the **Proof-of-Concept (PoC)** phase. Honest engineering comes first:

- [x] **Phase 1: Architectural Concept & Python Prototype** (`powershell32.py`, basic assembler & simulation scripts).
- [X] **Phase 2: Core Emulator Rewrite (In Progress)** — Porting the Python execution loop to **Rust / C** for deterministic performance.
- [ ] **Phase 3: Minimalist Value Product** — Linux Based system graphic with new "linux-os" for all and enterprise model
- [ ] **Phase 4: FPGA & Hardware Prototyping** — Testing decentralized module discovery on open FPGA boards.
- [ ] **Phase 5: Get working linux based system** - Graphical interface and most powerful tools with VOS
---

## 🤝 How to Contribute

MacroMag is an ambitious, founder-led open-source project initiated by Adrian Morawski (2026). We are looking for curious engineers, systems programmers, and hardware tinkerers who want to build something unconventional from the ground up.

## 🟡 Rule of MacroMag ™
Online But Always Updating, We giving two versions of MacroMag OS: Free for people, paid for enterprises. 

## ⚠️ Transparency & Security Notice (About `.exe` Files)

If you notice pre-compiled `.exe` binaries in this repository or in our Releases, please note that **they are legitimate, open-source system utilities and prototypes** built for the upcoming **MacroMag-32** ecosystem (for testing and emulation purposes).

* **100% Open Source:** Every binary comes directly from the source code available in this repository (licensed under **GNU GPL v3.0**). You do not have to trust our pre-built files—you can inspect the code and compile everything yourself from scratch.
* **Zero Telemetry & Safe:** In accordance with the MacroMag Foundation core principles, none of our tools contain telemetry, tracking, or obfuscated scripts.
* **Why `.exe`?** While our target architecture is a bare-metal RISC-V microkernel, early userspace apps, tools, and simulators are temporarily compiled for Windows to allow rapid testing and development.

If you have any questions about a specific file or want to verify a build, feel free to open an Issue or check the source directories!


We especially welcome help with:
1. **Systems Programming (Rust / C):** Rewriting the Python PoC runtime into a fast, memory-safe emulator.
2. **Linux Based system Architecture:** Designing the minimal IPC (Inter-Process Communication) and sandboxing layer.
3. **FPGA / Hardware Tinkering:** Prototyping decentralized device registration protocols.

Check the **Issues** tab to see current tasks, or open a Discussion to share architectural feedback.

---

## 💼 Licensing & Vision

MacroMag is licensed under **GPL v3** to ensure the core architecture remains free and open forever. Our long-term goal is to develop open-source developer kits and modular hardware appliances for industrial automation, education, and air-gapped security environments. Adrian Morawski Is Official Project Leader (like CEO) 

*Initiated by Adrian Morawski (2026)*
