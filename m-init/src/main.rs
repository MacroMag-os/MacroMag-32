use std::fs;
use std::io::{self, Write};
use std::process::Command;

fn mount_virtual_filesystems() {
    let _ = fs::create_dir_all("/proc");
    let _ = fs::create_dir_all("/sys");
    let _ = fs::create_dir_all("/dev");

    // Montowanie /proc i /sys, aby odczytywać prawdziwe dane o sprzęcie
    let _ = Command::new("/bin/mount").args(["-t", "proc", "none", "/proc"]).status();
    let _ = Command::new("/bin/mount").args(["-t", "sysfs", "none", "/sys"]).status();
}

fn print_banner() {
    print!("\x1B[2J\x1B[1;1H"); // Czyszczenie ekranu
    println!("\x1B[1;36m==============================================================\x1B[0m");
    println!("\x1B[1;37m   __  __                       __  __                        \x1B[0m");
    println!("\x1B[1;37m  |  \\/  | __ _  ___ _ __ ___  |  \\/  | __ _  __ _            \x1B[0m");
    println!("\x1B[1;37m  | |\\/| |/ _` |/ __| '__/ _ \\ | |\\/| |/ _` |/ _` |           \x1B[0m");
    println!("\x1B[1;37m  | |  | | (_| | (__| | | (_) || |  | | (_| | (_| |           \x1B[0m");
    println!("\x1B[1;37m  |_|  |_|\\__,_|\\___|_|  \\___/ |_|  |_|\\__,_|\\__, |           \x1B[0m");
    println!("\x1B[1;36m                                             |___/  OS v0.1   \x1B[0m");
    println!("\x1B[1;36m==============================================================\x1B[0m");
    println!("  Architecture : Pure Linux Kernel + VOS (Zero GNU Clutter)");
    println!("  Telemetry    : \x1B[1;32mDISABLED (100% Air-Gapped)\x1B[0m");
    println!("  Type \x1B[1;33mhelp\x1B[0m to explore Virtualized Object Shell commands.\n");
}

fn get_hardware_info() -> String {
    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let total_mem = meminfo
        .lines()
        .find(|l| l.starts_with("MemTotal:"))
        .unwrap_or("MemTotal: Unknown");

    let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let cpu_model = cpuinfo
        .lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split(':').nth(1))
        .unwrap_or(" Generic x86_64 CPU")
        .trim();

    format!(
        "{{ Object: SystemHardware, CPU: \"{}\", Memory: \"{}\", Telemetry: false }}",
        cpu_model,
        total_mem.replace("MemTotal:", "").trim()
    )
}

fn handle_command(cmd: &str) {
    let trimmed = cmd.trim();
    match trimmed {
        "" => {}
        "help" => {
            println!("\x1B[1;33mAvailable VOS Prototype Commands:\x1B[0m");
            println!("  \x1B[1;36msys.info()\x1B[0m                                        - Inspect live hardware object");
            println!("  \x1B[1;36mvm.list()\x1B[0m                                         - List active sandboxed micro-VMs");
            println!("  \x1B[1;36mvm.list() | filter status == \"running\"\x1B[0m            - Demonstrate typed object pipeline");
            println!("  \x1B[1;36mspawn --isolated --image:web_server -> Core.VM\x1B[0m    - Launch isolated sandbox process");
            println!("  \x1B[1;36mclear\x1B[0m                                             - Clear terminal screen");
            println!("  \x1B[1;36mreboot\x1B[0m                                            - Reboot machine");
        }
        "sys.info()" => {
            println!("\x1B[1;32m[Typed Object Stream]\x1B[0m");
            println!("{}", get_hardware_info());
        }
        "vm.list()" => {
            println!("\x1B[1;32m[Typed Object Stream: 3 Objects]\x1B[0m");
            println!("{{ id: 1, name: \"Core.VOS\", status: \"running\", memory_mb: 32, sandbox: true }}");
            println!("{{ id: 2, name: \"Net.ZeroTrust\", status: \"running\", memory_mb: 16, sandbox: true }}");
            println!("{{ id: 3, name: \"Legacy.Bridge\", status: \"stopped\", memory_mb: 0, sandbox: true }}");
        }
        "vm.list() | filter status == \"running\"" => {
            println!("\x1B[1;32m[Filtered Object Stream: 2 Objects]\x1B[0m");
            println!("{{ id: 1, name: \"Core.VOS\", status: \"running\", memory_mb: 32, sandbox: true }}");
            println!("{{ id: 2, name: \"Net.ZeroTrust\", status: \"running\", memory_mb: 16, sandbox: true }}");
        }
        "spawn --isolated --image:web_server -> Core.VM" => {
            println!("\x1B[1;35m[Sandbox Manager]\x1B[0m Allocating isolated memory namespace...");
            println!("\x1B[1;32m[OK]\x1B[0m Process 'web_server' bound to Core.VM (PID: 104, Telemetry: BLOCKED).");
        }
        "clear" => {
            print_banner();
        }
        "reboot" => {
            println!("Shutting down MacroMag OS...");
            let _ = fs::write("/proc/sysrq-trigger", "b");
        }
        _ => {
            println!("\x1B[1;31m[VOS SyntaxError]\x1B[0m Unknown object or method: '{}'. Type 'help'.", trimmed);
        }
    }
}

fn main() {
    mount_virtual_filesystems();
    print_banner();

    let stdin = io::stdin();
    loop {
        print!("\x1B[1;36mVOS > \x1B[0m");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if stdin.read_line(&mut input).is_ok() {
            handle_command(&input);
        }
    }
}
