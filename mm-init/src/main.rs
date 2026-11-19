use std::fs;
use std::io::{self, Write};
use std::process::Command;

fn mount_virtual_fs() {
    let _ = fs::create_dir_all("/proc");
    let _ = fs::create_dir_all("/sys");
    let _ = fs::create_dir_all("/dev");
    let _ = Command::new("/bin/busybox").args(["mount", "-t", "proc", "none", "/proc"]).status();
    let _ = Command::new("/bin/busybox").args(["mount", "-t", "sysfs", "none", "/sys"]).status();
    let _ = Command::new("/bin/busybox").args(["mount", "-t", "devtmpfs", "none", "/dev"]).status();
}

fn print_banner() {
    println!("\x1b[2J\x1b[H");
    println!("==========================================================");
    println!("  MM > MacroMag OS 2.0v (Zero-Telemetry Linux Core)");
    println!("  Architecture: Linux HAL + Virtualized Object Shell (VOS)");
    println!("==========================================================");
    println!("  Wpisz 'help' aby zobaczyc dostepne obiekty VOS.\n");
}

fn cmd_sys_info() {
    let version = fs::read_to_string("/proc/version").unwrap_or_else(|_| "Unknown Kernel".into());
    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let total_ram = meminfo.lines().next().unwrap_or("MemTotal: Unknown");
    println!("[System.Info Object]");
    println!("  OS      : MacroMag-32 / 2.0v (Community Edition)");
    println!("  Kernel  : {}", version.split_whitespace().take(3).collect::<Vec<_>>().join(" "));
    println!("  Memory  : {}", total_ram);
}

fn cmd_files_list(filter: Option<&str>) {
    println!("[Files.List Object Pipeline]");
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(f) = filter {
                if !name.contains(f) {
                    continue;
                }
            }
            let meta = entry.metadata().ok();
            let kind = if meta.as_ref().map(|m| m.is_dir()).unwrap_or(false) { "DIR " } else { "FILE" };
            let size = meta.map(|m| m.len()).unwrap_or(0);
            println!("  {{ type: \"{}\", name: \"{}\", size: {} B }}", kind, name, size);
        }
    }
}

fn main() {
    mount_virtual_fs();
    print_banner();

    let stdin = io::stdin();
    loop {
        print!("MM-VOS > ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() {
            continue;
        }
        let cmd = input.trim();

        if cmd.is_empty() {
            continue;
        } else if cmd == "help" {
            println!("Dostepne potoki obiektowe VOS:");
            println!("  sys.info()                  - Pobiera obiekt z informacjami o sprzecie i jadrze");
            println!("  files.list()                - Zwraca liste obiektow w biezacym katalogu");
            println!("  files.list() | filter <txt> - Filtruje obiekty po nazwie");
            println!("  clear                       - Czysci ekran");
            println!("  reboot                      - Restartuje maszyne");
        } else if cmd == "sys.info()" {
            cmd_sys_info();
        } else if cmd == "files.list()" {
            cmd_files_list(None);
        } else if cmd.starts_with("files.list() | filter ") {
            let pattern = cmd.trim_start_matches("files.list() | filter ").trim().trim_matches('"');
            cmd_files_list(Some(pattern));
        } else if cmd == "clear" {
            print_banner();
        } else if cmd == "reboot" {
            println!("Zamykanie MacroMag OS...");
            let _ = Command::new("/bin/busybox").args(["reboot", "-f"]).status();
        } else {
            println!("VOS Error: Nieznany obiekt lub metoda '{}'. Wpisz 'help'.", cmd);
        }
    }
}
