# Contributing to MacroMag

Thank you for your interest in contributing to **MacroMag**! We welcome systems programmers, Rust/C developers, RISC-V enthusiasts, hardware engineers, and technical writers who share our mission of digital sovereignty and e-waste reduction.

---

## 1. Areas Where We Need Help
* **Systems Programming (Rust / C):** Rewriting and optimizing the core execution runtime and emulator.
* **64-bit RISC-V Microkernel:** Designing minimal IPC (Inter-Process Communication), memory management, and sandboxing layers.
* **Virtualized Object Shell (VOS):** Developing typed object pipelines and native sandbox syntax.
* **Open Hardware & FPGA:** Prototyping decentralized module descriptors (Phantom BIOS) and magnetic/pogo-pin interconnects.

---

## 2. How to Submit Changes
1. **Fork the repository** and create a new branch from `main` (e.g., `feat/riscv-ipc` or `fix/vos-parser`).
2. **Keep changes focused:** Submit one logical feature or bug fix per Pull Request (PR).
3. **Code Standards:**
   * Ensure all new source files include the standard SPDX license header at the top:
     ```c
     // SPDX-License-Identifier: GPL-3.0-or-later
     // Copyright (C) 2026 MacroMag Contributors
     ```
   * Write clean, well-documented code with zero telemetry or external network calls.
4. **Open a Pull Request** describing what your change does and why it is needed.

---

## 3. Legal Requirement: Developer Certificate of Origin (DCO)
To protect the legal integrity of the MacroMag codebase, **all contributors must certify that they have the right to submit their code** under the project's **GNU GPL v3.0** license.

We enforce the **Developer Certificate of Origin (DCO) v1.1** (the same standard used by the Linux Kernel). By submitting a Pull Request, you certify the following:

> **Developer Certificate of Origin — Version 1.1**
>
> By making a contribution to this project, I certify that:
> (a) The contribution was created in whole or in part by me and I have the right to submit it under the open source license indicated in the file; or
> (b) The contribution is based upon previous work that, to the best of my knowledge, is covered under an appropriate open source license and I have the right under that license to submit that work with modifications, whether created in whole or in part by me, under the same open source license (unless I am permitted to submit under a different license), as indicated in the file; or
> (c) The contribution was provided directly to me by some other person who certified (a), (b) or (c) and I have not modified it.
> (d) I understand and agree that this project and the contribution are public and that a record of the contribution (including all personal information I submit with it, including my sign-off) is maintained indefinitely and may be redistributed consistent with this project or the open source license(s) involved.

### How to Sign Off Your Commits
Every commit in your Pull Request **must** include a `Signed-off-by` line with your real name (or legal pseudonym) and email address. Git can add this automatically when you use the `-s` flag:

```bash
git commit -s -m "feat: add initial RISC-V UART driver"
```

This adds the following line to your commit message:
```text
Signed-off-by: Your Name <your.email@example.com>
```

Pull Requests containing unsigned commits cannot be merged into `main`.

---

## 4. Governance & Community Conduct
* All contributions are reviewed by the **Core Maintainer Team** as defined in `GOVERNANCE.md`.
* Treat all contributors with respect and focus on constructive technical feedback.
