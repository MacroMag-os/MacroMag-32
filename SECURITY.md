# Security Policy — MacroMag

The **MacroMag** Core Maintainer Team takes the security of our 64-bit RISC-V microkernel, Virtualized Object Shell (VOS), and execution runtimes seriously. We appreciate the security community's efforts in responsibly disclosing vulnerabilities.

---

## 1. Supported Versions
MacroMag is currently in active Proof-of-Concept (PoC) and R&D development. Security updates are applied directly to the `main` branch:

| Component / Branch | Status | Security Support |
| :--- | :--- | :--- |
| `main` (Core Microkernel & VOS) | Active Development | Supported |
| Legacy 32-bit Python Prototypes | Archived / Experimental | Best-effort only |

---

## 2. Reporting a Vulnerability (Responsible Disclosure)
**Please do NOT report security vulnerabilities through public GitHub issues, discussions, or pull requests.**

If you discover a security vulnerability, memory safety issue, sandbox escape, or cryptographic flaw in MacroMag:

1. **Contact the Security & Legal Lead directly:** Send an encrypted or plain-text report to **[macromagfoundation@gmail.com]** (Attention: Alan Morawski, Security & Legal Lead).
2. **Use GitHub Private Vulnerability Reporting:** Alternatively, you may submit a confidential report via the **Security -> Advisories -> Report a vulnerability** tab in this repository.
3. **Include the following details:**
   * Description of the vulnerability and affected component (e.g., RISC-V microkernel, VOS parser, Rust/C runtime).
   * Step-by-step instructions or a Proof-of-Concept (PoC) script to reproduce the issue.
   * Potential impact assessment.

---

## 3. Response Timeline
* **Acknowledgment:** Within **48–72 hours** of receiving your report.
* **Assessment & Triage:** Within **7 days**, we will confirm the vulnerability and determine its severity.
* **Remediation & Disclosure:** Once a patch is developed and verified, we will coordinate public disclosure and credit the researcher (unless anonymity is requested).

---

## 4. Security Architecture Principles
* **Zero Telemetry:** MacroMag never collects or transmits user data, crash dumps, or hardware identifiers without explicit local user action.
* **Memory Safety:** Core system components are actively migrated to memory-safe **Rust** alongside audited **C** primitives.
