mod vos_builtins;
mod vos_core;
mod vos_executor;
mod vos_parser;

use std::ffi::CString;
use std::fs;
use std::io::{self, Write};
use vos_core::VosObject;
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

fn main() {
    init_hardware_environment();

    print!("\x1B[2J\x1B[1;1H");
    println!("==================================================");
    println!("   MacroMag OS x86-64 | Virtualized Object Shell  ");
    println!("   Zero Telemetry | Pure Rust/C Bare-Metal Linux  ");
    println!("==================================================");
    println!("Wpisz 'help' aby zobaczyc komendy.\n");

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
            println!("Dostepne komendy VOS (x86-64):");
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
                            Err(e) => println!("[VOS Error]: {}", e),
                        }
                    }
                    Err(e) => println!("[Blad skladni]: {}", e),
                }
            }
            Err(e) => println!("[Blad leksera]: {}", e),
        }
    }
}

fn print_table(rows: &[VosObject]) {
    if rows.is_empty() {
        println!("(brak wynikow)");
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
