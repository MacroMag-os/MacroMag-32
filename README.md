 # MacroMag: Modular Offline Computing Architecture

[![License: GPL v3](https://shields.io)](https://gnu.org)
[![Status: Experimental](https://shields.io)]()

> *"Those who are crazy enough to think they can change the world, are the ones who do."* — Steve Jobs

Modern computing is trapped in a cycle of planned obsolescence. Mainstream operating systems and locked-down hardware (*eFUSEs*, remote kill-switches, mandatory cloud telemetry) actively turn perfectly functional electronics into e-waste.

**MacroMag is an experimental hardware-software architecture designed for digital sovereignty, modularity, and zero-telemetry online execution.**

Instead of patching forty-year-old legacy standards, we are building a clean, minimalist ecosystem focused on object-oriented command execution, hardware-level sandboxing, and modular hardware design.

---

## ⚡ Core Architectural Pillars

### 1. The Virtualized Object Shell (VOS)
MacroMag replaces traditional raw-text shells with an object-oriented execution environment.
* **Typed Memory Pipelines:** Commands exchange strongly-typed objects directly in memory rather than parsing fragile text streams (`grep`/`awk`/`sed` hacks).
* **Native Sandbox Syntax:** Process isolation is built directly into the shell syntax. Every untrusted execution runs inside a lightweight microkernel sandbox by default.

### 2. Minimalist Bare-Metal Microkernel (RISC-V Target)
* **Zero Telemetry, 100% Determinism:** No background cloud polling, no hidden corporate firmware. CPU cycles go strictly into user-defined tasks.
* **Open Architecture First:** Rather than reinventing proprietary silicon, MacroMag targets open **64-bit RISC-V** hardware and FPGA implementations, leveraging existing modern toolchains (Rust/C/LLVM).
* **Self-Contained Core:** Designed to operate completely air-gapped with a lean, auditable set of essential system tools.

### 3. Long-Term R&D: Magnetic Modularity & Phantom BIOS
Our long-term hardware vision aims to eliminate single-point-of-failure motherboards and fight e-waste:
* **Decentralized Phantom BIOS:** Each physical hardware module carries its own minimal firmware descriptor, registering its capabilities dynamically upon connection.
* **Hot-Swappable Hardware Nodes:** Researching fault-tolerant, magnetic/pogo-pin interconnects for deterministic real-time module replacement without system crashes.

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
- [ ] **Phase 2: Core Emulator Rewrite (In Progress)** — Porting the Python execution loop to **Rust / C** for deterministic performance.
- [ ] **Phase 3: Minimalist Microkernel** — Booting a basic VOS prototype on QEMU (RISC-V 64-bit).
- [ ] **Phase 4: FPGA & Hardware Prototyping** — Testing decentralized module discovery on open FPGA boards.

---

## 🤝 How to Contribute

MacroMag is an ambitious, founder-led open-source project initiated by Adrian Morawski (2026). We are looking for curious engineers, systems programmers, and hardware tinkerers who want to build something unconventional from the ground up.

## 🟡 Rule of MacroMag ™
Online But Always Updating

We especially welcome help with:
1. **Systems Programming (Rust / C):** Rewriting the Python PoC runtime into a fast, memory-safe emulator.
2. **RISC-V & Microkernel Architecture:** Designing the minimal IPC (Inter-Process Communication) and sandboxing layer.
3. **FPGA / Hardware Tinkering:** Prototyping decentralized device registration protocols.

Check the **Issues** tab to see current tasks, or open a Discussion to share architectural feedback.

---

## 💼 Licensing & Vision

MacroMag is licensed under **GPL v3** to ensure the core architecture remains free and open forever. Our long-term goal is to develop open-source developer kits and modular hardware appliances for industrial automation, education, and air-gapped security environments.

*Initiated by Adrian Morawski (2026)*
