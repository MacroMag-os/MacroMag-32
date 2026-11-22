mod vos_builtins;
mod vos_core;
mod vos_executor;
mod vos_parser;

use std::collections::BTreeSet;
use std::ffi::CString;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use vos_core::{VosObject, VosValue};
use vos_parser::{tokenize, Parser};

const SYS_MOUNT_X86_64: i64 = 165;

extern "C" {
    fn syscall(number: i64, ...) -> i64;
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

fn init_hardware_environment() {
    mount_kernel_fs("devtmpfs", "/dev", "devtmpfs");
    mount_kernel_fs("proc", "/proc", "proc");
    mount_kernel_fs("sysfs", "/sys", "sysfs");
    mount_kernel_fs("tmpfs", "/tmp", "tmpfs");
}

/// Podnosi interfejsy sieciowe i pobiera IP przez DHCP w tle (dla komendy mur.sync())
fn init_network() {
    let _ = Command::new("/bin/ip").args(["link", "set", "lo", "up"]).status();

    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let iface = entry.file_name().to_string_lossy().to_string();
            if iface != "lo" {
                let _ = Command::new("/bin/ip")
                    .args(["link", "set", &iface, "up"])
                    .status();
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
    init_hardware_environment();
    init_network();

    print!("\x1B[2J\x1B[1;1H");
    println!("\x1B[1;36m==================================================\x1B[0m");
    println!("\x1B[1;37m   MacroMag OS x86-64 | Virtualized Object Shell  \x1B[0m");
    println!("\x1B[0;36m   Zero Telemetry | Pure Rust/C Bare-Metal Linux  \x1B[0m");
    println!("\x1B[1;36m==================================================\x1B[0m");
    println!("Wpisz \x1B[1;33mhelp\x1B[0m aby zobaczyc komendy.\n");

    loop {
        print!("\x1B[1;32mvos>\x1B[0m ");
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
            println!("\x1B[1;36mDostepne komendy VOS (x86-64):\x1B[0m");
            println!("  sys.ps()                - lista procesow z /proc");
            println!("  sys.mem()               - zuzycie pamieci RAM");
            println!("  sys.uptime()            - czas dzialania systemu");
            println!("  fs.ls(\"/dev\")           - wykryte urzadzenia");
            println!("  fs.cat(\"/proc/cpuinfo\") - informacje o procesorze");
            println!("  mur.sync() / mur.list() - menedzer pakietow MUR");
            println!("  sys.reboot()            - restart komputera");
            println!("  sys.poweroff()          - wylaczenie zasilania");
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
                            Err(e) => println!("\x1B[1;31m[VOS Error]:\x1B[0m {}", e),
                        }
                    }
                    Err(e) => println!("\x1B[1;31m[Blad skladni]:\x1B[0m {}", e),
                }
            }
            Err(e) => println!("\x1B[1;31m[Blad leksera]:\x1B[0m {}", e),
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

/// Renderuje obiekty VosObject w postaci czytelnej, kolorowej tabeli z wyrownanymi kolumnami
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

    // Naglowek tabeli
    let header: Vec<String> = cols
        .iter()
        .enumerate()
        .map(|(i, c)| format!("\x1B[1;36m{:<width$}\x1B[0m", c.to_uppercase(), width = widths[i]))
        .collect();
    println!("  {}", header.join(" | "));

    // Linia oddzielajaca
    let sep: Vec<String> = widths.iter().map(|&w| "-".repeat(w)).collect();
    println!("  \x1B[0;90m{}\x1B[0m", sep.join("-+-"));

    // Wiersze danych
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
