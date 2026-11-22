use crate::vos_core::{VosObject, VosValue};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

const SYS_REBOOT_X86_64: i64 = 169;
const LINUX_REBOOT_MAGIC1: u64 = 0xfee1dead;
const LINUX_REBOOT_MAGIC2: u64 = 672274793;
const LINUX_REBOOT_CMD_RESTART: u64 = 0x01234567;
const LINUX_REBOOT_CMD_POWER_OFF: u64 = 0x4321fedc;

extern "C" {
    fn syscall(number: i64, ...) -> i64;
}

pub fn execute_builtin(module: &str, action: &str, args: &[VosValue]) -> Result<Vec<VosObject>, String> {
    match (module, action) {
        ("sys", "ps") => Ok(sys_ps()),
        ("sys", "mem") => Ok(sys_mem()),
        ("sys", "uptime") => Ok(sys_uptime()),
        ("sys", "install") => {
            let Some(VosValue::Text(disk)) = args.first() else {
                return Err("sys.install() wymaga sciezki dysku, np. sys.install(\"/dev/sda\")".into());
            };
            sys_install(disk)
        }
        ("sys", "reboot") => {
            println!("Restartowanie MacroMag OS...");
            unsafe {
                syscall(SYS_REBOOT_X86_64, LINUX_REBOOT_MAGIC1, LINUX_REBOOT_MAGIC2, LINUX_REBOOT_CMD_RESTART, 0u64);
            }
            Ok(Vec::new())
        }
        ("sys", "poweroff") => {
            println!("Wylaczanie zasilania...");
            unsafe {
                syscall(SYS_REBOOT_X86_64, LINUX_REBOOT_MAGIC1, LINUX_REBOOT_MAGIC2, LINUX_REBOOT_CMD_POWER_OFF, 0u64);
            }
            Ok(Vec::new())
        }
        ("fs", "ls") => {
            let path = match args.first() {
                Some(VosValue::Text(p)) => p.as_str(),
                None => "/",
                _ => return Err("fs.ls() wymaga sciezki tekstowej".into()),
            };
            fs_ls(path)
        }
        ("fs", "cat") => {
            let Some(VosValue::Text(path)) = args.first() else {
                return Err("fs.cat() wymaga sciezki pliku".into());
            };
            fs_cat(path)
        }
        ("mur", "sync") => run_external("/usr/bin/mur", &["sync"]),
        ("mur", "list") => run_external("/usr/bin/mur", &["list"]),
        ("mur", "install") => {
            let Some(VosValue::Text(pkg)) = args.first() else {
                return Err("mur.install() wymaga nazwy pakietu".into());
            };
            run_external("/usr/bin/mur", &["install", pkg.as_str()])
        }
        ("sys", "exec") => {
            let Some(VosValue::Text(bin)) = args.first() else {
                return Err("sys.exec() wymaga nazwy programu".into());
            };
            let str_args: Vec<String> = args.iter().skip(1).map(|v| match v {
                VosValue::Text(s) => s.clone(),
                VosValue::Int(i) => i.to_string(),
                VosValue::Float(f) => f.to_string(),
                VosValue::Bytes(b) => b.to_string(),
                VosValue::Bool(b) => b.to_string(),
            }).collect();
            let refs: Vec<&str> = str_args.iter().map(|s| s.as_str()).collect();
            run_external(bin, &refs)
        }
        _ => Err(format!("Nieznana komenda: {}.{}()", module, action)),
    }
}

fn sys_install(disk: &str) -> Result<Vec<VosObject>, String> {
    if !disk.starts_with("/dev/") {
        return Err("Nieprawidlowa sciezka urzadzenia (oczekiwano np. /dev/sda)".into());
    }
    if fs::metadata(disk).is_err() {
        return Err(format!("Nie znaleziono dysku '{}' w /dev", disk));
    }

    println!("\x1B[1;33m[INSTALLER]\x1B[0m Partycjonowanie dysku {}...", disk);
    let part_script = "label: dos\n,,L,*\n";
    let mut sfdisk = Command::new("sfdisk")
        .arg(disk)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|_| "Brak narzedzia 'sfdisk' w obrazie ISO".to_string())?;

    if let Some(mut stdin) = sfdisk.stdin.take() {
        let _ = stdin.write_all(part_script.as_bytes());
    }
    let status = sfdisk.wait().map_err(|e| format!("Blad partycjonowania: {}", e))?;
    if !status.success() {
        return Err("Partycjonowanie dysku nie powiodlo sie".into());
    }

    let part = if disk.contains("nvme") {
        format!("{}p1", disk)
    } else {
        format!("{}1", disk)
    };

    println!("\x1B[1;33m[INSTALLER]\x1B[0m Formatowanie {} (ext4)...", part);
    let mkfs = Command::new("mkfs.ext4")
        .args(["-F", "-L", "MACROMAG_ROOT", &part])
        .status()
        .map_err(|_| "Brak narzedzia 'mkfs.ext4' w obrazie ISO".to_string())?;
    if !mkfs.success() {
        return Err("Formatowanie ext4 nie powiodlo sie".into());
    }

    let _ = fs::create_dir_all("/mnt/target");
    let mnt = Command::new("/bin/mount")
        .args([&part, "/mnt/target"])
        .status()
        .map_err(|e| format!("Blad montowania {}: {}", part, e))?;
    if !mnt.success() {
        return Err("Nie udalo sie zamontowac partycji docelowej".into());
    }

    println!("\x1B[1;33m[INSTALLER]\x1B[0m Kopiowanie plikow MacroMag OS na dysk...");
    for dir in ["boot/grub", "usr/bin", "etc", "var/cache/mur", "proc", "sys", "dev", "tmp"] {
        let _ = fs::create_dir_all(format!("/mnt/target/{}", dir));
    }

    let _ = fs::copy("/init", "/mnt/target/init");
    let _ = fs::copy("/usr/bin/mur", "/mnt/target/usr/bin/mur");
    if fs::metadata("/boot/vmlinuz").is_ok() {
        let _ = fs::copy("/boot/vmlinuz", "/mnt/target/boot/vmlinuz");
    }
    if fs::metadata("/boot/initramfs.img").is_ok() {
        let _ = fs::copy("/boot/initramfs.img", "/mnt/target/boot/initramfs.img");
    }

    let grub_cfg = "set timeout=1\nset default=0\n\nmenuentry \"MacroMag OS (Installed)\" {\n    linux /boot/vmlinuz root=/dev/sda1 console=tty0 quiet\n    initrd /boot/initramfs.img\n}\n";
    let _ = fs::write("/mnt/target/boot/grub/grub.cfg", grub_cfg);

    println!("\x1B[1;33m[INSTALLER]\x1B[0m Instalacja bootloadera GRUB na {}...", disk);
    let grub_ok = Command::new("grub-install")
        .args(["--boot-directory=/mnt/target/boot", disk])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let mut res = HashMap::new();
    res.insert("target_disk".into(), VosValue::Text(disk.to_string()));
    res.insert("partition".into(), VosValue::Text(part));
    res.insert("grub_installed".into(), VosValue::Bool(grub_ok));
    res.insert("status".into(), VosValue::Text("DONE - Wyjmij pendrive i wpisz sys.reboot()".into()));

    Ok(vec![res])
}

fn sys_ps() -> Vec<VosObject> {
    let mut list = Vec::new();
    let Ok(entries) = fs::read_dir("/proc") else { return list };
    for entry in entries.flatten() {
        let Ok(name_os) = entry.file_name().into_string() else { continue };
        let Ok(pid) = name_os.parse::<i64>() else { continue };
        let proc_name = fs::read_to_string(format!("/proc/{}/comm", pid))
            .unwrap_or_else(|_| "kernel".into()).trim().to_string();
        let mut rss_bytes: u64 = 0;
        if let Ok(status_txt) = fs::read_to_string(format!("/proc/{}/status", pid)) {
            for line in status_txt.lines() {
                if let Some(rest) = line.strip_prefix("VmRSS:") {
                    if let Some(kb) = rest.split_whitespace().next().and_then(|s| s.parse::<u64>().ok()) {
                        rss_bytes = kb * 1024;
                    }
                    break;
                }
            }
        }
        let mut obj = HashMap::new();
        obj.insert("pid".into(), VosValue::Int(pid));
        obj.insert("name".into(), VosValue::Text(proc_name));
        obj.insert("memory".into(), VosValue::Bytes(rss_bytes));
        list.push(obj);
    }
    list
}

fn sys_mem() -> Vec<VosObject> {
    let mut list = Vec::new();
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines().take(5) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Some(Ok(kb)) = parts.get(1).map(|s| s.parse::<u64>()) {
                    let mut obj = HashMap::new();
                    obj.insert("metric".into(), VosValue::Text(parts.first().unwrap_or(&"").trim_end_matches(':').into()));
                    obj.insert("size".into(), VosValue::Bytes(kb * 1024));
                    list.push(obj);
                }
            }
        }
    }
    list
}

fn sys_uptime() -> Vec<VosObject> {
    let mut obj = HashMap::new();
    let raw = fs::read_to_string("/proc/uptime").unwrap_or_else(|_| "0.0".into());
    let secs = raw.split_whitespace().next().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
    obj.insert("seconds".into(), VosValue::Int(secs as i64));
    vec![obj]
}

fn fs_ls(path: &str) -> Result<Vec<VosObject>, String> {
    let entries = fs::read_dir(path).map_err(|e| format!("Blad {}: {}", path, e))?;
    let mut list = Vec::new();
    for entry in entries.flatten() {
        let mut obj = HashMap::new();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let (kind, size) = match entry.metadata() {
            Ok(m) => (if m.is_dir() { "dir" } else { "file" }, m.len()),
            Err(_) => ("unknown", 0),
        };
        obj.insert("name".into(), VosValue::Text(file_name));
        obj.insert("type".into(), VosValue::Text(kind.into()));
        obj.insert("size".into(), VosValue::Bytes(size));
        list.push(obj);
    }
    Ok(list)
}

fn fs_cat(path: &str) -> Result<Vec<VosObject>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Blad {}: {}", path, e))?;
    Ok(content.lines().enumerate().map(|(i, line)| {
        let mut obj = HashMap::new();
        obj.insert("line".into(), VosValue::Int((i + 1) as i64));
        obj.insert("text".into(), VosValue::Text(line.to_string()));
        obj
    }).collect())
}

fn run_external(program: &str, args: &[&str]) -> Result<Vec<VosObject>, String> {
    let output = Command::new(program).args(args).output()
        .map_err(|e| format!("Blad uruchomienia '{}': {}", program, e))?;
let stdout = String::from_utf8_lossy(&output.stdout);
let stderr = String::from_utf8_lossy(&output.stderr);
if !output.status.success() && !stderr.is_empty() {
return Err(stderr.trim().to_string());
}
Ok(stdout.lines().enumerate().map(|(i, line)| {
let mut obj = HashMap::new();
obj.insert("line".into(), VosValue::Int((i + 1) as i64));
obj.insert("output".into(), VosValue::Text(line.to_string()));
obj
}).collect())
}
