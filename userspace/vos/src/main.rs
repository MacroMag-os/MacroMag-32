mod vos_builtins;
mod vos_core;
mod vos_executor;
mod vos_parser;

use std::ffi::CString;
use std::fs;
use std::io::{self, Write};
use vos_core::VosObject;
use vos_parser::{tokenize, Parser};

// Bezpośredni numer wywołania systemowego `mount` w architekturze Linux x86-64
const SYS_MOUNT_X86_64: i64 = 165;

extern "C" {
    fn syscall(number: i64, ...) -> i64;
}

/// Montuje wirtualne systemy plików jądra na fizycznym sprzęcie x86-64 bez użycia zewnętrznego /bin/mount
fn mount_kernel_fs(source: &str, target: &str, fstype: &str) {
    let _ = fs::create_dir_all(target);
    let src = CString::new(source).unwrap();
    let tgt = CString::new(target).unwrap();
    let fs_t = CString::new(fstype).unwrap();

    unsafe {
        syscall(
            SYS_MOUNT_X86_64,
            src.as_ptr(),
            tgt.as_ptr(),
            fs_t.as_ptr(),
            0u64,
            std::ptr::null::<u8>(),
        );
    }
}

fn init_hardware_environment() {
    // 1. Montujemy /dev (klawiatura, ekran tty, dyski), /proc (procesy) i /sys (sprzęt)
    mount_kernel_fs("devtmpfs", "/dev", "devtmpfs");
    mount_kernel_fs("proc", "/proc", "proc");
    mount_kernel_fs("sysfs", "/sys", "sysfs");
    mount_kernel_fs("tmpfs", "/tmp", "tmpfs");
}

fn main() {
    // Uruchamiamy inicjalizację sprzętu (działa, gdy VOS jest PID 1)
    init_hardware_environment();

    // Czyszczenie ekranu VGA/Framebuffer i baner powitalny MacroMag OS
    print!("\x1B[2J\x1B[1;1H");
    println!("==================================================");
    println!("   MacroMag OS x86-64 | Virtualized Object Shell  ");
    println!("   Zero Telemetry | Pure Rust/C Bare-Metal Linux  ");
    println!("==================================================");
    println!("Wpisz 'help' aby zobaczyć komendy lub 'sys.poweroff()' aby wyłączyć.\n");

    loop {
        print!("vos> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "clear" {
            print!("\x1B[2J\x1B[1;1H");
            continue;
        }

        if trimmed == "help" {
            println!("Dostępne komendy VOS (x86-64):");
            println!("  sys.ps()              - lista procesów z /proc");
            println!("  sys.mem()             - zużycie pamięci RAM");
            println!("  sys.uptime()          - czas działania systemu");
            println!("  fs.ls(\"/dev\")         - wykryte urządzenia sprzętowe");
            println!("  fs.cat(\"/proc/cpuinfo\") - informacje o procesorze x86-64");
            println!("  mur.sync() / mur.list() - menedżer pakietów MUR");
            println!("  sys.reboot()          - restart komputera");
            println!("  sys.poweroff()        - bezpieczne wyłączenie zasilania");
            continue;
        }

        match tokenize(trimmed) {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                match parser.parse_pipeline() {
                    Ok(pipeline) => {
                        match vos_builtins::execute_builtin(
                            &pipeline.source.module,
                            &pipeline.source.action,
                            &pipeline.source.args,
                        ) {
                            Ok(data) => {
                                let results = pipeline.execute(data);
                                print_table(&results);
                            }
                            Err(e) => println!("[VOS Error]: {}", e),
                        }
                    }
                    Err(e) => println!("[Błąd składni]: {}", e),
                }
            }
            Err(e) => println!("[Błąd leksera]: {}", e),
        }
    }
}

fn print_table(rows: &[VosObject]) {
    if rows.is_empty() {
        println!("(brak wyników)");
        return;
    }
    for row in rows {
        let line: Vec<String> = row
            .iter()
            .map(|(k, v)| format!("{}: {:?}", k, v))
            .collect();
        println!("  {}", line.join(" | "));
    }
    println!();
}

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

#[derive(Debug, Clone)]
struct FileObj {
    name: String,
    is_dir: bool,
    size: u64,
}

fn run_installer() {
    println!("\x1b[1;33m=== MacroMag OS Disk Installer (mm-install) ===\x1b[0m");
    println!("WARNING: This will format /dev/sda and install MacroMag OS!");
    print!("Type 'YES' to continue: ");
    io::stdout().flush().unwrap();

    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).unwrap();
    if confirm.trim() != "YES" {
        println!("Installation aborted.");
        return;
    }

    println!("[1/4] Formatting /dev/sda as ext4...");
    let status = Command::new("/sbin/mkfs.ext4").args(["-F", "/dev/sda"]).status();
    if status.is_err() || !status.unwrap().success() {
        println!("\x1b[1;31mError: Could not format /dev/sda (check if virtual disk is attached).\x1b[0m");
        return;
    }

    println!("[2/4] Mounting target disk to /mnt...");
    let _ = fs::create_dir_all("/mnt");
    let _ = Command::new("/bin/mount").args(["/dev/sda", "/mnt"]).status();

    println!("[3/4] Copying MacroMag root filesystem to disk...");
    for dir in ["bin", "sbin", "etc", "usr", "root", "boot"] {
        let src = format!("/{}", dir);
        let _ = Command::new("/bin/cp").args(["-a", &src, "/mnt/"]).status();
    }

    println!("[4/4] Installing GRUB bootloader to /dev/sda...");
    let _ = Command::new("/sbin/grub-install")
        .args(["--boot-directory=/mnt/boot", "/dev/sda"])
        .status();

    println!("\x1b[1;32mInstallation complete! Type 'reboot' and remove the ISO.\x1b[0m");
}

fn main() {
    // Automatyczne wyświetlenie fastfetch przy starcie sesji
    let _ = Command::new("/usr/bin/fastfetch")
        .args(["--logo", "/etc/fastfetch_logo.txt"])
        .status();

    loop {
        let cwd = env::current_dir().unwrap_or_else(|_| "/".into());
        print!("\x1b[1;35mVOS\x1b[0m \x1b[1;34m{}\x1b[0m > ", cwd.display());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).unwrap_or(0) == 0 {
            break;
        }
        let line = input.trim();
        if line.is_empty() {
            continue;
        }

        // Obsługa obiektowego potoku VOS: np. fs.list() | filter is_dir == true
        if line.starts_with("fs.list()") {
            let mut objects: Vec<FileObj> = Vec::new();
            if let Ok(entries) = fs::read_dir(&cwd) {
                for entry in entries.flatten() {
                    if let Ok(meta) = entry.metadata() {
                        objects.push(FileObj {
                            name: entry.file_name().to_string_lossy().into_owned(),
                            is_dir: meta.is_dir(),
                            size: meta.len(),
                        });
                    }
                }
            }

            if line.contains("filter is_dir == true") {
                objects.retain(|o| o.is_dir);
            }

            println!("{:<25} {:<10} {:<10}", "NAME", "TYPE", "SIZE (B)");
            println!("{}", "-".repeat(45));
            for obj in objects {
                println!(
                    "{:<25} {:<10} {:<10}",
                    obj.name,
                    if obj.is_dir { "DIR" } else { "FILE" },
                    obj.size
                );
            }
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0] {
            "exit" | "logout" => break,
            "cd" => {
                let target = parts.get(1).copied().unwrap_or("/root");
                if let Err(e) = env::set_current_dir(target) {
                    eprintln!("cd error: {}", e);
                }
            }
            "mkdir" => {
                if let Some(dir_name) = parts.get(1) {
                    if let Err(e) = fs::create_dir_all(dir_name) {
                        eprintln!("mkdir error: {}", e);
                    }
                } else {
                    println!("Usage: mkdir <directory_name>");
                }
            }
            "mm-install" => run_installer(),
            "reboot" => {
                let _ = Command::new("/sbin/reboot").args(["-f"]).status();
            }
            cmd => {
                // Uruchamianie programów zewnętrznych (fastfetch, ls, ip, dropbear, itp.)
                match Command::new(cmd).args(&parts[1..]).status() {
                    Ok(_) => {}
                    Err(_) => println!("VOS: unknown command or object '{}'", cmd),
                }
            }
        }
    }
}
