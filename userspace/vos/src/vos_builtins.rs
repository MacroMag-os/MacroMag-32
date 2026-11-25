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

extern "C" { fn syscall(number: i64, ...) -> i64; }

pub fn execute_builtin(module: &str, action: &str, args: &[VosValue]) -> Result<Vec<VosObject>, String> {
    match (module, action) {
        ("sys", "ps") => Ok(sys_ps()),
        ("sys", "mem") => Ok(sys_mem()),
        ("sys", "uptime") => Ok(sys_uptime()),
        ("sys", "cpu") => Ok(sys_cpu()),
        ("disk", "list") => Ok(disk_list()),
        ("net", "interfaces") => Ok(net_interfaces()),
        ("sys", "install") => {
            let Some(VosValue::Text(disk)) = args.first() else { return Err("Podaj dysk, np. sys.install(\"/dev/sda\")".into()); };
            sys_install(disk)
        }
        ("sys", "reboot") => {
            unsafe { syscall(SYS_REBOOT_X86_64, LINUX_REBOOT_MAGIC1, LINUX_REBOOT_MAGIC2, LINUX_REBOOT_CMD_RESTART, 0u64); }
            Ok(Vec::new())
        }
        ("sys", "poweroff") => {
            unsafe { syscall(SYS_REBOOT_X86_64, LINUX_REBOOT_MAGIC1, LINUX_REBOOT_MAGIC2, LINUX_REBOOT_CMD_POWER_OFF, 0u64); }
            Ok(Vec::new())
        }
        ("fs", "ls") => fs_ls(match args.first() { Some(VosValue::Text(p)) => p.as_str(), None => "/", _ => return Err("Zla sciezka".into()) }),
        ("fs", "cat") => { let Some(VosValue::Text(p)) = args.first() else { return Err("Podaj plik".into()); }; fs_cat(p) }
        ("mur", "sync") => run_external("/usr/bin/mur", &["sync"]),
        ("mur", "list") => run_external("/usr/bin/mur", &["list"]),
        ("mur", "install") => { let Some(VosValue::Text(p)) = args.first() else { return Err("Podaj pakiet".into()); }; run_external("/usr/bin/mur", &["install", p.as_str()]) }
        _ => Err(format!("Unknown komenda: {}.{}()", module, action)),
    }
}
fn sys_install(disk: &str) -> Result<Vec<VosObject>, String> {
    if !disk.starts_with("/dev/") || fs::metadata(disk).is_err() { return Err(format!("Missing dysku {}", disk)); }
    println!("[INSTALLER] Partycjonowanie {}...", disk);
    let mut fd = Command::new("/bin/fdisk").arg(disk).stdin(Stdio::piped()).stdout(Stdio::null()).spawn().map_err(|_| "Missing fdisk".to_string())?;
    if let Some(mut s) = fd.stdin.take() { let _ = s.write_all(b"o\nn\np\n1\n\n\na\nw\n"); }
    if !fd.wait().map(|s| s.success()).unwrap_or(false) { return Err("Error fdisk".into()); }
    let part = if disk.contains("nvme") || disk.contains("mmcblk") { format!("{}p1", disk) } else { format!("{}1", disk) };
    std::thread::sleep(std::time::Duration::from_secs(1));
    if !Command::new("mkfs.ext4").args(["-F", "-L", "MACROMAG_ROOT", &part]).status().map(|s| s.success()).unwrap_or(false) {
        if !Command::new("/bin/mkfs.ext2").args(["-F", "-L", "MACROMAG_ROOT", &part]).status().map(|s| s.success()).unwrap_or(false) { return Err("Error mkfs".into()); }
    }
    let _ = fs::create_dir_all("/mnt/target");
    if !Command::new("/bin/mount").args([&part, "/mnt/target"]).status().map(|s| s.success()).unwrap_or(false) { return Err("Error mount".into()); }
    for d in ["boot/grub", "bin", "sbin", "usr/bin", "usr/sbin", "etc", "root", "var/cache/mur", "proc", "sys", "dev", "dev/pts", "tmp"] { let _ = fs::create_dir_all(format!("/mnt/target/{}", d)); }
    let _ = fs::copy("/init", "/mnt/target/init");
    for d in ["bin", "sbin", "usr", "etc"] { let _ = Command::new("/bin/cp").args(["-a", &format!("/{}/.", d), &format!("/mnt/target/{}/", d)]).status(); }
    let _ = fs::create_dir_all("/mnt/cdrom");
    for dev in ["/dev/sr0", "/dev/sdb1", "/dev/sdb", "/dev/vdb"] {
        if Command::new("/bin/mount").args(["-o", "ro", dev, "/mnt/cdrom"]).status().map(|s| s.success()).unwrap_or(false) {
            if fs::metadata("/mnt/cdrom/boot/vmlinuz").is_ok() {
                let _ = fs::copy("/mnt/cdrom/boot/vmlinuz", "/mnt/target/boot/vmlinuz");
                let _ = fs::copy("/mnt/cdrom/boot/initramfs.img", "/mnt/target/boot/initramfs.img");
                let _ = Command::new("/bin/umount").arg("/mnt/cdrom").status();
                break;
            }
            let _ = Command::new("/bin/umount").arg("/mnt/cdrom").status();
        }
    }
    let cfg = format!("set timeout=2\nset default=0\nmenuentry \"MacroMag OS\" {{\n    linux /boot/vmlinuz root={} rw init=/init console=ttyS0 console=tty0 quiet\n    initrd /boot/initramfs.img\n}}\n", part);
    let _ = fs::write("/mnt/target/boot/grub/grub.cfg", cfg);
    let grub_ok = Command::new("grub-install").args(["--target=i386-pc", "--boot-directory=/mnt/target/boot", disk]).status().map(|s| s.success()).unwrap_or(false);
    let _ = Command::new("/bin/umount").arg("/mnt/target").status();
    let mut res = HashMap::new();
    res.insert("disk".into(), VosValue::Text(disk.to_string()));
    res.insert("grub".into(), VosValue::Bool(grub_ok));
    res.insert("status".into(), VosValue::Text("DONE".into()));
    Ok(vec![res])
}
fn sys_ps() -> Vec<VosObject> {
    let mut list = Vec::new();
    let Ok(entries) = fs::read_dir("/proc") else { return list };
    for e in entries.flatten() {
        let Ok(pid) = e.file_name().to_string_lossy().parse::<i64>() else { continue };
        let name = fs::read_to_string(format!("/proc/{}/comm", pid)).unwrap_or_else(|_| "kernel".into()).trim().to_string();
        let mut obj = HashMap::new();
        obj.insert("pid".into(), VosValue::Int(pid));
        obj.insert("name".into(), VosValue::Text(name));
        list.push(obj);
    }
    list
}
fn sys_mem() -> Vec<VosObject> {
    let mut list = Vec::new();
    if let Ok(c) = fs::read_to_string("/proc/meminfo") {
        for l in c.lines().take(4) {
            let p: Vec<&str> = l.split_whitespace().collect();
            if p.len() >= 2 { if let Ok(kb) = p[1].parse::<u64>() { let mut o = HashMap::new(); o.insert("metric".into(), VosValue::Text(p[0].trim_end_matches(":").into())); o.insert("size".into(), VosValue::Bytes(kb * 1024)); list.push(o); } }
        }
    }
    list
}
fn sys_uptime() -> Vec<VosObject> {
    let mut list = Vec::new();
    if let Ok(c) = fs::read_to_string("/proc/uptime") {
        let s = c.split_whitespace().next().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
        let mut o = HashMap::new(); o.insert("seconds".into(), VosValue::Int(s as i64)); list.push(o);
    }
    list
}
fn fs_ls(path: &str) -> Result<Vec<VosObject>, String> {
    let entries = fs::read_dir(path).map_err(|e| format!("Blad: {}", e))?;
    let mut list = Vec::new();
    for e in entries.flatten() {
        let mut o = HashMap::new();
        o.insert("name".into(), VosValue::Text(e.file_name().to_string_lossy().to_string()));
        o.insert("size".into(), VosValue::Bytes(e.metadata().map(|m| m.len()).unwrap_or(0)));
        list.push(o);
    }
    Ok(list)
}
fn fs_cat(path: &str) -> Result<Vec<VosObject>, String> {
    let c = fs::read_to_string(path).map_err(|e| format!("Blad: {}", e))?;
    Ok(c.lines().enumerate().map(|(i, l)| { let mut o = HashMap::new(); o.insert("line".into(), VosValue::Int((i+1) as i64)); o.insert("text".into(), VosValue::Text(l.to_string())); o }).collect())
}
fn run_external(prog: &str, args: &[&str]) -> Result<Vec<VosObject>, String> {
    let out = Command::new(prog).args(args).output().map_err(|e| format!("Error {}: {}", prog, e))?;
    if !out.status.success() { return Err(String::from_utf8_lossy(&out.stderr).trim().to_string()); }
    Ok(String::from_utf8_lossy(&out.stdout).lines().enumerate().map(|(i, l)| { let mut o = HashMap::new(); o.insert("line".into(), VosValue::Int((i+1) as i64)); o.insert("output".into(), VosValue::Text(l.to_string())); o }).collect())
}

fn disk_list() -> Vec<VosObject> {
    let mut list = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/block") else { return list };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with("loop") || name.starts_with("ram") { continue; }
        let sectors: u64 = fs::read_to_string(format!("/sys/block/{}/size", name)).unwrap_or_default().trim().parse().unwrap_or(0);
        let model = fs::read_to_string(format!("/sys/block/{}/device/model", name)).unwrap_or_else(|_| "Virtual/Block".into()).trim().to_string();
        let mut o = HashMap::new();
        o.insert("device".into(), VosValue::Text(format!("/dev/{}", name)));
        o.insert("model".into(), VosValue::Text(model));
        o.insert("size".into(), VosValue::Bytes(sectors * 512));
        list.push(o);
    }
    list
}

fn net_interfaces() -> Vec<VosObject> {
    let mut list = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/net") else { return list };
    for e in entries.flatten() {
        let iface = e.file_name().to_string_lossy().to_string();
        let state = fs::read_to_string(format!("/sys/class/net/{}/operstate", iface)).unwrap_or_else(|_| "unknown".into()).trim().to_string();
        let mac = fs::read_to_string(format!("/sys/class/net/{}/address", iface)).unwrap_or_else(|_| "-".into()).trim().to_string();
        let rx: u64 = fs::read_to_string(format!("/sys/class/net/{}/statistics/rx_bytes", iface)).unwrap_or_default().trim().parse().unwrap_or(0);
        let mut o = HashMap::new();
        o.insert("interface".into(), VosValue::Text(iface));
        o.insert("state".into(), VosValue::Text(state));
        o.insert("mac".into(), VosValue::Text(mac));
        o.insert("rx".into(), VosValue::Bytes(rx));
        list.push(o);
    }
    list
}

fn sys_cpu() -> Vec<VosObject> {
    let mut list = Vec::new();
    if let Ok(c) = fs::read_to_string("/proc/cpuinfo") {
        let mut model = String::from("Unknown CPU");
        let mut cores = 0i64;
        for l in c.lines() {
            if l.starts_with("model name") { model = l.split(":").nth(1).unwrap_or("").trim().to_string(); }
            if l.starts_with("processor") { cores += 1; }
        }
        let mut o = HashMap::new();
        o.insert("model".into(), VosValue::Text(model));
        o.insert("cores".into(), VosValue::Int(cores));
        list.push(o);
    }
    list
}
