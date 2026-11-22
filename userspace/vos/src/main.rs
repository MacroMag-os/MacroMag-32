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

const SYS_MOUNT_X86_64: i64 = 165;
const SIGINT: i32 = 2;
const SIG_IGN: usize = 1;

extern "C" {
    fn syscall(number: i64, ...) -> i64;
    fn signal(sig: i32, handler: usize) -> usize;
}

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

fn init_system_environment() {
    // 1. Ignorujemy Ctrl+C (SIGINT) w PID 1, zeby przerwanie programu nie powodowalo Kernel Panic
    unsafe {
        signal(SIGINT, SIG_IGN);
    }

    // 2. Montujemy wirtualne systemy plikow jadra
    mount_kernel_fs("devtmpfs", "/dev", "devtmpfs");
    mount_kernel_fs("proc", "/proc", "proc");
    mount_kernel_fs("sysfs", "/sys", "sysfs");
    mount_kernel_fs("tmpfs", "/tmp", "tmpfs");

    // 3. Ustawiamy standardowe zmienne srodowiskowe jak w Ubuntu / Termux
    env::set_var("PATH", "/bin:/sbin:/usr/bin:/usr/sbin");
    env::set_var("HOME", "/root");
    env::set_var("TERM", "linux");
    env::set_var("USER", "root");
    env::set_var("HOSTNAME", "macromag");

    let _ = fs::create_dir_all("/root");
    let _ = fs::create_dir_all("/etc");
    let _ = env::set_current_dir("/root");

    // 4. Konfigurujemy DNS (Cloudflare + Google), zeby dzialal internet po nazwach domen
    let _ = fs::write("/etc/resolv.conf", "nameserver 1.1.1.1\nnameserver 8.8.8.8\n");

    // 5. Podnosimy karty sieciowe i DHCP w tle
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
    println!("Wpisz standardowa komende Linuxa (np. ls, fastfetch, mur) lub potok VOS (np. sys.ps()).\n");

    loop {
        // Znak zachety jak w Ubuntu / Termux: root@macromag:/sciezka#
        let cwd = env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "/".into());
        let display_cwd = if cwd == "/root" { "~".to_string() } else { cwd };

        print!("\x1B[1;32mroot@macromag\x1B[0m:\x1B[1;34m{}\x1B[0m# ", display_cwd);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        // 1. Wbudowane komendy powloki (cd, clear, exit, help)
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
            println!("Srodowisko MacroMag OS obsluguje dwa tryby pracy:");
            println!("  1. Standardowe komendy Linuxa: ls, cd, cat, mkdir, rm, ip, fastfetch, mur");
            println!("  2. Potoki obiektowe VOS:      sys.ps() | where memory > 10MB | select pid, name");
            println!("  3. Zarzadzanie systemem:      sys.install(\"/dev/sda\"), sys.reboot(), sys.poweroff()");
            continue;
        }

        // 2. Rozpoznanie: czy to potok obiektowy VOS (ma nawiasy '()' lub potok '|'), czy zwykla komenda Linuxa?
        let is_vos_syntax = trimmed.contains('(') || trimmed.contains('|');

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
                        Err(e) => println!("\x1B[1;31m[Blad skladni VOS]:\x1B[0m {}", e),
                    }
                }
                Err(e) => println!("\x1B[1;31m[Blad leksera]:\x1B[0m {}", e),
            }
        } else {
            // 3. Uruchomienie standardowego programu Linuxa w pelnym trybie interaktywnym (jak Bash)
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            let prog = parts;
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
        .map(|(i, c)| format!("\x1B[1;36m{:<width$}\x1B[0m", c.to_uppercase(), width = widths[i]))
        .collect();
    println!("  {}", header.join(" | "));

    let sep: Vec<String> = widths.iter().map(|&w| "-".repeat(w)).collect();
    println!("  \x1B[0;90m{}\x1B[0m", sep.join("-+-"));

    for r in formatted_rows {
        let line: Vec<String> = r
            .iter()
            .enumerate()
            .map(|(i, cell)| format!("{:<width$}", cell, width = widths[i]))
            .collect();
        println!("  {}", line.join(" | "));
    }
    println!();
}
