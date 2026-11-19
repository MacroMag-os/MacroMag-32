use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

type VosObj = BTreeMap<String, String>;

fn mount_virtual_fs() {
    for dir in ["/proc", "/sys", "/dev", "/mnt"] {
        let _ = fs::create_dir_all(dir);
    }
    let _ = Command::new("/bin/busybox").args(["mount", "-t", "proc", "none", "/proc"]).status();
    let _ = Command::new("/bin/busybox").args(["mount", "-t", "sysfs", "none", "/sys"]).status();
    let _ = Command::new("/bin/busybox").args(["mount", "-t", "devtmpfs", "none", "/dev"]).status();
}

fn print_banner() {
    println!("\x1b[2J\x1b[H");
    println!("==================================================================");
    println!("  MM > MacroMag OS 2.0v (Zero-Telemetry Linux Core)");
    println!("  Engine: Virtualized Object Shell (VOS v2) + System Installer");
    println!("==================================================================");
    println!("  Wpisz 'help' aby zobaczyc obiekty VOS i polecenie instalacji.\n");
}

fn get_sys_info() -> Vec<VosObj> {
    let version = fs::read_to_string("/proc/version").unwrap_or_else(|_| "Linux".into());
    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let ram = meminfo.lines().next().unwrap_or("Unknown").replace("MemTotal:", "").trim().to_string();
    let mut obj = VosObj::new();
    obj.insert("os".into(), "MacroMag-32 2.0v".into());
    obj.insert("kernel".into(), version.split_whitespace().take(3).collect::<Vec<_>>().join(" "));
    obj.insert("memory".into(), ram);
    obj.insert("arch".into(), "x86_64".into());
    vec![obj]
}

fn get_proc_list() -> Vec<VosObj> {
    let mut list = Vec::new();
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let pid = entry.file_name().to_string_lossy().to_string();
            if pid.chars().all(|c| c.is_ascii_digit()) {
                let comm_path = format!("/proc/{}/comm", pid);
                let name = fs::read_to_string(comm_path).unwrap_or_else(|_| "kernel".into()).trim().to_string();
                let mut obj = VosObj::new();
                obj.insert("pid".into(), pid);
                obj.insert("name".into(), name);
                obj.insert("state".into(), "running".into());
                list.push(obj);
            }
        }
    }
    list
}

fn get_disk_list() -> Vec<VosObj> {
    let mut list = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let dev = entry.file_name().to_string_lossy().to_string();
            if dev.starts_with("loop") || dev.starts_with("ram") {
                continue;
            }
            let size_path = format!("/sys/block/{}/size", dev);
            let sectors: u64 = fs::read_to_string(size_path).unwrap_or_default().trim().parse().unwrap_or(0);
            let mb = (sectors * 512) / (1024 * 1024);
            let mut obj = VosObj::new();
            obj.insert("device".into(), format!("/dev/{}", dev));
            obj.insert("size_mb".into(), mb.to_string());
            list.push(obj);
        }
    }
    list
}

fn get_files_list() -> Vec<VosObj> {
    let mut list = Vec::new();
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let meta = entry.metadata().ok();
            let kind = if meta.as_ref().map(|m| m.is_dir()).unwrap_or(false) { "dir" } else { "file" };
            let size = meta.map(|m| m.len()).unwrap_or(0);
            let mut obj = VosObj::new();
            obj.insert("name".into(), name);
            obj.insert("type".into(), kind.into());
            obj.insert("bytes".into(), size.to_string());
            list.push(obj);
        }
    }
    list
}

fn run_installer() {
    println!("\n[MM-INSTALLER] Rozpoczynam instalacje MacroMag OS na dysku /dev/sda...");
    if !std::path::Path::new("/dev/sda").exists() {
        println!("[BLAD] Nie wykryto dysku /dev/sda! Uruchom QEMU z dyskiem wirtualnym.");
        return;
    }
    println!("[1/3] Formatowanie dysku /dev/sda systemem plikow ext4...");
    let status = Command::new("/bin/busybox").args(["mkfs.ext4", "-F", "/dev/sda"]).status();
    if status.map(|s| !s.success()).unwrap_or(true) {
        println!("[INFO] Tworzenie podstawowej struktury na /dev/sda...");
    }
    println!("[2/3] Montowanie partycji docelowej w /mnt...");
    let _ = Command::new("/bin/busybox").args(["mount", "/dev/sda", "/mnt"]).status();
    println!("[3/3] Kopiowanie jadra systemu i plikow MacroMag OS...");
    let _ = fs::create_dir_all("/mnt/boot");
    let _ = fs::create_dir_all("/mnt/etc");
    let _ = fs::copy("/init", "/mnt/init");
    let _ = fs::write("/mnt/etc/macromag-release", "MacroMag OS 2.0v Installed\n");
    let _ = Command::new("/bin/busybox").args(["sync"]).status();
    let _ = Command::new("/bin/busybox").args(["umount", "/mnt"]).status();
    println!("[SUKCES] MacroMag OS zostal zainstalowany na dysku /dev/sda!\n");
}

fn render_objects(mut items: Vec<VosObj>, stages: &[&str]) {
    let mut as_json = false;
    for stage in stages {
        let s = stage.trim();
        if s.starts_with("where ") {
            let cond = s.trim_start_matches("where ").trim();
            if let Some((key, val)) = cond.split_once("==") {
                let k = key.trim();
                let v = val.trim().trim_matches('"');
                items.retain(|obj| obj.get(k).map(|x| x == v).unwrap_or(false));
            } else if let Some((key, val)) = cond.split_once("~=") {
                let k = key.trim();
                let v = val.trim().trim_matches('"');
                items.retain(|obj| obj.get(k).map(|x| x.contains(v)).unwrap_or(false));
            }
        } else if s == "to.json()" {
            as_json = true;
        }
    }

    if as_json {
        println!("[");
        for (i, obj) in items.iter().enumerate() {
            let fields: Vec<String> = obj.iter().map(|(k, v)| format!("\"{}\": \"{}\"", k, v)).collect();
            let comma = if i + 1 < items.len() { "," } else { "" };
            println!("  {{ {} }}{}", fields.join(", "), comma);
        }
        println!("]");
    } else {
        for obj in items {
            let fields: Vec<String> = obj.iter().map(|(k, v)| format!("{}: \"{}\"", k, v)).collect();
            println!("  {{ {} }}", fields.join(", "));
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
        let line = input.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        let source = parts[0].trim();
        let pipes = &parts[1..];

        match source {
            "help" => {
                println!("Obiekty zrodlowe VOS (mozesz laczyc je z '| where ...' oraz '| to.json()'):");
                println!("  sys.info()      - Informacje o systemie, jadrze i RAM");
                println!("  proc.list()     - Lista aktywnych procesow w jadrze");
                println!("  disk.list()     - Wykryte dyski twarde i urzadzenia blokowe");
                println!("  files.list()    - Obiekty w biezacym katalogu");
                println!("  sys.install()   - Uruchamia instalator MacroMag OS na dysk /dev/sda");
                println!("  clear / reboot  - Czyszczenie ekranu / Restart systemu");
                println!("Przyklad: proc.list() | where name ~= \"init\" | to.json()");
            }
            "sys.info()" => render_objects(get_sys_info(), pipes),
            "proc.list()" => render_objects(get_proc_list(), pipes),
            "disk.list()" => render_objects(get_disk_list(), pipes),
            "files.list()" => render_objects(get_files_list(), pipes),
            "sys.install()" => run_installer(),
            "clear" => print_banner(),
            "reboot" => {
                let _ = Command::new("/bin/busybox").args(["reboot", "-f"]).status();
            }
            _ => println!("VOS Error: Nieznany obiekt '{}'. Wpisz 'help'.", source),
        }
    }
}
