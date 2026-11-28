# MUR (MacroMag User Repository)

**MUR** (*MacroMag User Repository*) is the native package manager and software repository ecosystem designed specifically for MacroMag OS.

## Why MUR?

Standard package managers often introduce heavy dependency trees and large metadata databases that slow down older storage media and consume excessive memory. MUR is built to remain fast, transparent, and predictable on constrained hardware.

## Key Capabilities

* **Lightweight Footprint:** Minimal disk and RAM usage during dependency resolution and package extraction.
* **GNU-Free Packaging:** Every package in MUR is built and verified to work seamlessly within the MacroMag OS Rust/C userland without expecting GNU libraries or systemd units.
* **Community & Extensibility:** Designed to allow developers to easily package and distribute lightweight software tailored for legacy systems.

## Common Commands

```bash
# Synchronize package database
mur sync

# Install a new package
mur install <package-name>

# Remove an installed package
mur remove <package-name>
```
