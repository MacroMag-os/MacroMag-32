# Virtualized Object Shell (VOS)

## 1. Abstract & Architectural Overview

The **Virtualized Object Shell (VOS)** is the primary userspace execution environment and `PID 1` initialization runtime of **MacroMag OS**. Engineered from the ground up in **Rust** with zero external crate dependencies, VOS replaces traditional POSIX/GNU text-based command-line interpreters (such as `bash`, `sh`, `grep`, and `awk`) with a **strongly-typed, object-oriented memory pipeline** combined with **native kernel-level process sandboxing**.

By eliminating the computational overhead of text serialization, stream parsing, and legacy GNU userland binaries, VOS achieves deterministic execution, minimal memory footprint (~400 KB static binary), and immediate responsiveness on both modern `x86_64` systems and resource-constrained legacy hardware.

---

## 2. Paradigm Shift: Raw Text Streams vs. Typed Object Pipelines

### The POSIX Bottleneck (Traditional Shells)
In classical Unix/Linux architectures, inter-process communication (IPC) across shell pipelines (`|`) relies on unstructured byte streams (`stdout` / `stdin`). Every utility in a pipeline must:
1. Serialize internal data structures into human-readable ASCII/UTF-8 text.
2. Transmit the text across kernel pipe buffers.
3. Re-parse that text using fragile regular expressions or delimiter heuristics (`awk '{print $3}'`, `sed`, `cut`).

This approach introduces high CPU overhead, localization bugs, and security vulnerabilities caused by improper string escaping.

### The VOS Solution (In-Memory Structured Objects)
In VOS, commands do not emit raw text—they yield vectors of strongly-typed key-value structures (`Vec<VosObject>`) directly in memory. Each field retains its native data type (`Int`, `Float`, `Bytes`, `Bool`, `Text`) throughout the entire execution lifecycle.

| Feature | Traditional POSIX Shell (`bash`) | MacroMag VOS |
| :--- | :--- | :--- |
| **Data Transport** | Unstructured raw text (`char*` streams) | Strongly-typed in-memory objects (`VosObject`) |
| **Filtering & Querying** | External text parsers (`grep`, `awk`, `sed`) | Built-in AST query operators (`where`, `select`, `limit`) |
| **Unit Awareness** | Manual string-to-integer conversion | Native unit literals (`B`, `KB`, `MB`, `GB`) parsed at AST time |
| **Process Isolation** | External wrappers (`unshare`, `firejail`, Docker) | First-class syntax keyword (`sandbox(...) ->`) |
| **Init System Role** | Separate daemon (`systemd`, `OpenRC`) | Native `PID 1` runtime mounting `/dev`, `/proc`, `/sys` directly |

---

## 3. Formal Grammar & Execution Pipeline

VOS processes user input through a deterministic three-stage compiler-like pipeline:

1. **Lexical Analyzer (`vos_parser::tokenize`):** Converts raw input into a stream of strongly-typed tokens, automatically resolving memory unit literals (e.g., `256MB` → `VosValue::Bytes(268435456)`).
2. **Recursive Descent Parser (`vos_parser::Parser`):** Constructs an Abstract Syntax Tree (`VosPipeline`) enforcing standard operator precedence (logical `&&` binds tighter than `||`).
3. **In-Memory Executor (`vos_executor` & `vos_builtins`):** Interfaces directly with Linux kernel virtual filesystems (`/proc`, `/sys`, `/dev`) and executes short-circuit logical filtering in RAM without spawning sub-shells.

### Syntax Example
```vos
// Query active processes consuming over 256 MB of RAM inside an isolated network sandbox
sandbox(net: false) -> sys.ps() | where memory > 256MB && status == "running" | select pid, name, memory | limit 10
```

---

## 4. Native Sandbox Execution Model

Security in VOS is not an afterthought—it is embedded directly into the shell grammar. Prefixing any pipeline with `sandbox(...) ->` instructs the VOS runtime to invoke Linux kernel namespace syscalls (`CLONE_NEWNET`, `CLONE_NEWNS`, `CLONE_NEWPID`) prior to task execution. Untrusted tools or packages installed via **MUR (MacroMag User Repository)** can be executed with zero network access and a read-only filesystem without requiring heavy container runtimes.

---

## 5. Summary of Core Built-in Modules

* **`sys.*` (Kernel & Process Telemetry):** Direct interface to `/proc` and Linux power-management syscalls (`sys.ps()`, `sys.mem()`, `sys.uptime()`, `sys.exec()`, `sys.reboot()`, `sys.poweroff()`).
* **`fs.*` (Deterministic Filesystem Operations):** Structured directory and file inspection (`fs.ls()`, `fs.cat()`).
* **`mur.*` (Package Orchestration):** Native integration with the **MacroMag User Repository** (`mur.sync()`, `mur.list()`, `mur.install()`).
