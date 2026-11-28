# VOS (Virtualized Object Shell)

**VOS** (*Virtualized Object Shell*) is the heart of MacroMag OS. Written in Rust, it serves a dual purpose: it acts as the system's **init process (PID 1)** and provides an interactive, object-oriented command-line environment.

## Key Features

* **Direct PID 1 Execution:** Eliminates the overhead of separate init daemons by managing system initialization, process spawning, and signal handling directly within the shell architecture.
* **Object-Oriented Pipeline:** Unlike traditional POSIX shells (such as Bash) that pass raw unstructured text streams between commands, VOS treats system resources, processes, and outputs as structured objects.
* **Memory Safety:** Leveraging Rust's ownership model ensures high stability and eliminates common memory-related vulnerabilities at the lowest level of userland.

## Basic Usage & Concepts

*(Note: Update this section with your exact VOS syntax examples as the shell evolves.)*

* **System Inspection:** Query active processes and hardware states directly as objects.
* **Service Management:** Spawn and monitor background tasks directly from the PID 1 supervisor.
* **Scripting:** Automate system tasks using native object properties rather than text parsing (`grep`/`awk`/`sed`).
