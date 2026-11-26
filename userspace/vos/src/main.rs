mod vos_builtins;
mod vos_core;
mod vos_executor;
mod vos_parser;

use std::collections::BTreeSet;
use std::env;
use std::ffi::CString;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use vos_core::{VosObject, VosValue};
use vos_parser::{tokenize, Parser};

const SIGINT: i32 = 2;
const SIG_IGN: usize = 1;
const WNOHANG: i32 = 1;
const O_RDWR: i32 = 2;

extern "C" {
    fn signal(sig: i32, handler: usize) -> usize;
    fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    fn mount(
        source: *const std::ffi::c_char,
        target: *const std::ffi::c_char,
        filesystemtype: *const std::ffi::c_char,
        mountflags: usize,
        data: *const core::ffi::c_void,
    ) -> i32;
    fn setsid() -> i32;
    fn open(pathname: *const std::ffi::c_char, flags: i32, ...) -> i32;
    fn dup2(oldfd: i32, newfd: i32) -> i32;
    fn close(fd: i32) -> i32;
}

fn attach_console() {
    let Ok(console) = CString::new("/dev/console") else { return; };
    unsafe {
        setsid();
        let fd = open(console.as_ptr().cast(), O_RDWR);
        if fd >= 0 {
            dup2(fd, 0);
            dup2(fd, 1);
            dup2(fd, 2);
            if fd > 2 {
                close(fd);
            }
        }
    }
}

fn reap_zombie_processes() {
    let mut status: i32 = 0;
    unsafe {
        while waitpid(-1, &mut status, WNOHANG) > 0 {}
    }
}

fn mount_kernel_fs(source: &str, target: &str, fstype: &str) {
    let _ = fs::create_dir_all(target);
    let Ok(src) = CString::new(source) else { return; };
    let Ok(tgt) = CString::new(target) else { return; };
    let Ok(fs_t) = CString::new(fstype) else { return; };

    unsafe {
        mount(
            src.as_ptr().cast(),
            tgt.as_ptr().cast(),
            fs_t.as_ptr().cast(),
            0,
            std::ptr::null(),
        );
    }
}

fn init_system_environment() {
    // 1. Ignorujemy Ctrl+C (SIGINT) w PID 1, zeby przerwanie programu nie powodowalo Kernel Panic
    unsafe {
        signal(SIGINT, SIG_IGN);
    }

    // 2. Montujemy wirtualne systemy plikow jadra (w tym /dev/pts dla SSH i terminali)
    mount_kernel_fs("devtmpfs", "/dev", "devtmpfs");
    attach_console();
    mount_kernel_fs("devpts", "/dev/pts", "devpts");
    mount_kernel_fs("proc", "/proc", "proc");
    mount_kernel_fs("sysfs", "/sys", "sysfs");
    mount_kernel_fs("tmpfs", "/tmp", "tmpfs");

    // 3. Ustawiamy standardowe zmienne srodowiskowe jak w Ubuntu Server
    env::set_var("PATH", "/bin:/sbin:/usr/bin:/usr/sbin");
    env::set_var("HOME", "/root");
    env::set_var("TERM", "linux");
    env::set_var("USER", "root");
    env::set_var("HOSTNAME", "macromag");

    let _ = fs::create_dir_all("/root");
    let _ = fs::create_dir_all("/etc");
    let _ = fs::create_dir_all("/var/log");
    let _ = env::set_current_dir("/root");

    // 4. Uruchamiamy logowanie jadra i systemu (syslogd / klogd)
    if fs::metadata("/bin/syslogd").is_ok() {
        let _ = Command::new("/bin/syslogd").spawn();
    }
    if fs::metadata("/bin/klogd").is_ok() {
        let _ = Command::new("/bin/klogd").spawn();
    }

    // 5. Konfigurujemy DNS (Cloudflare + Google)
    let _ = fs::write("/etc/resolv.conf", "nameserver 1.1.1.1\nnameserver 8.8.8.8\n");

    // 6. Podnosimy karty sieciowe i DHCP w tle
    let _ = Command::new("/bin/ip").args(["link", "set", "lo", "up"]).status();
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let iface = entry.file_name().to_string_lossy().to_string();
            if iface != "lo" {
                let _ = Command::new("/bin/ip").args(["link", "set", &iface, "up"]).status();
                let _ = Command::new("/bin/udhcpc")
                    .args(["-i", &iface, "-b", "-q"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
            }
        }
    }
}

fn main() {
    init_system_environment();

    print!("\x1B[2J\x1B[1;1H");
    println!("\x1B[1;36mMacroMag OS 0.1 LTS (x86_64) tty1\x1B[0m");
    println!("Kernel: Linux | Runtime: VOS + POSIX Userspace | Zero Telemetry\n");
    println!("Enter a Linux command (e.g. ls, dmesg | grep usb, mur) or a VOS object pipeline (e.g. sys.ps()).\n");

    loop {
        reap_zombie_processes();

        let cwd = env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "/".into());
        let display_cwd = if cwd == "/root" { "~".to_string() } else { cwd };

        print!("\x1B[1;32mroot@macromag\x1B[0m:\x1B[1;34m{}\x1B[0m# ", display_cwd);
        let _ = io::stdout().flush();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) | Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            Ok(_) => {}
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        // 1. Wbudowane komendy powloki
        if trimmed == "clear" {
            print!("\x1B[2J\x1B[1;1H");
            continue;
        }
        if trimmed == "cd" || trimmed.starts_with("cd ") {
            let target = trimmed.strip_prefix("cd").unwrap().trim();
            let dest = if target.is_empty() || target == "~" { "/root" } else { target };
            if let Err(e) = env::set_current_dir(dest) {
                println!("cd: {}: {}", dest, e);
            }
            continue;
        }
        if trimmed == "help" {
            println!("MacroMag OS supports two execution modes:");
            println!("  1. Standard Linux commands: ls, cd, cat, dmesg | grep ..., mur");
            println!("  2. VOS object pipelines:      sys.ps() | where memory > 10MB | select pid, name");
            println!("  3. Zarzadzanie systemem:      sys.install(\"/dev/sda\"), sys.reboot(), sys.poweroff()");
            continue;
        }

        // 2. Rozpoznanie: potok VOS (ma '(' przed pierwszym '|') vs klasyczna komenda/potok POSIX
        let first_segment = trimmed.split('|').next().unwrap_or("");
        let is_vos_syntax = first_segment.contains('(');

        if is_vos_syntax {
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
                                Err(e) => println!("\x1B[1;31m[VOS Error]:\x1B[0m {}", e),
                            }
                        }
                        Err(e) => println!("\x1B[1;31m[VOS Syntax Error]:\x1B[0m {}", e),
                    }
                }
                Err(e) => println!("\x1B[1;31m[VOS Lexer Error]:\x1B[0m {}", e),
            }
        } else if trimmed.contains('|') || trimmed.contains('>') || trimmed.contains('<') || trimmed.ends_with('&') || trimmed.contains(';') {
            // Klasyczne potoki uniksowe, przekierowania i zadania w tle delegujemy do /bin/sh
            let _ = Command::new("/bin/sh").args(["-c", trimmed]).status();
        } else {
            // Bezposrednie uruchomienie pojedynczego programu
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let Some(&prog) = parts.first() else { continue; };
            let args = &parts[1..];

            match Command::new(prog).args(args).status() {
                Ok(_) => {}
                Err(e) => {
                    if e.kind() == io::ErrorKind::NotFound {
                        println!("{}: command not found", prog);
                    } else {
                        println!("{}: {}", prog, e);
                    }
                }
            }
        }
    }
}

fn format_value(val: &VosValue) -> String {
    match val {
        VosValue::Text(s) => s.clone(),
        VosValue::Int(i) => i.to_string(),
        VosValue::Float(f) => format!("{:.2}", f),
        VosValue::Bool(b) => b.to_string(),
        VosValue::Bytes(b) => {
            if *b >= 1024 * 1024 * 1024 {
                format!("{:.1} GB", *b as f64 / (1024.0 * 1024.0 * 1024.0))
            } else if *b >= 1024 * 1024 {
                format!("{:.1} MB", *b as f64 / (1024.0 * 1024.0))
            } else if *b >= 1024 {
                format!("{:.1} KB", *b as f64 / 1024.0)
            } else {
                format!("{} B", b)
            }
        }
    }
}

fn print_table(rows: &[VosObject]) {
    if rows.is_empty() {
        println!("\x1B[0;90m(brak wynikow)\x1B[0m");
        return;
    }

    let mut col_set = BTreeSet::new();
    for row in rows {
        for key in row.keys() {
            col_set.insert(key.clone());
        }
    }
    let cols: Vec<String> = col_set.into_iter().collect();

    let mut widths: Vec<usize> = cols.iter().map(|c| c.len()).collect();
    let formatted_rows: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            cols.iter()
                .enumerate()
                .map(|(i, col)| {
                    let cell = row.get(col).map(format_value).unwrap_or_else(|| "-".into());
                    if cell.len() > widths[i] {
                        widths[i] = cell.len();
                    }
                    cell
                })
                .collect()
        })
        .collect();

    let header: Vec<String> = cols
        .iter()
        .enumerate()
        .map(|(i, c)| format!("\x1B[1;33m{:<width$}\x1B[0m", c.to_uppercase(), width = widths[i]))
        .collect();
    println!("{}", header.join(" | "));

    let sep: Vec<String> = widths.iter().map(|&w| "-".repeat(w)).collect();
    println!("{}", sep.join("-+-"));

    for row in formatted_rows {
        let line: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| format!("{:<width$}", cell, width = widths[i]))
            .collect();
        println!("{}", line.join(" | "));
    }
}
