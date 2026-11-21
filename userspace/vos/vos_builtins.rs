use crate::vos_core::{VosObject, VosValue};
use std::collections::HashMap;
use std::fs;

/// Główny router komend wywoływany w Twoim obecnym main.rs
pub fn execute_builtin(module: &str, action: &str, args: &[VosValue]) -> Result<Vec<VosObject>, String> {
    match (module, action) {
        ("sys", "ps") => Ok(sys_ps()),
        ("sys", "mem") => Ok(sys_mem()),
        ("sys", "uptime") => Ok(sys_uptime()),
        ("fs", "ls") => {
            let path = match args.first() {
                Some(VosValue::Text(p)) => p.as_str(),
                None => "/", // Domyślnie katalog główny, jeśli wpiszesz fs.ls()
                _ => return Err("fs.ls() wymaga ścieżki tekstowej, np. fs.ls(\"/etc\")".into()),
            };
            fs_ls(path)
        }
        ("fs", "cat") => {
            let path = match args.first() {
                Some(VosValue::Text(p)) => p.as_str(),
                _ => return Err("fs.cat() wymaga ścieżki pliku, np. fs.cat(\"/proc/version\")".into()),
            };
            fs_cat(path)
        }
        _ => Err(format!("Nieznana komenda systemowa: {}.{}()", module, action)),
    }
}

/// Czyta procesy z /proc wraz ze zużyciem pamięci RAM (VmRSS)
fn sys_ps() -> Vec<VosObject> {
    let mut list = Vec::new();
    let Ok(entries) = fs::read_dir("/proc") else { return list };

    for entry in entries.flatten() {
        let Ok(name_os) = entry.file_name().into_string() else { continue };
        let Ok(pid) = name_os.parse::<i64>() else { continue };

        let proc_name = fs::read_to_string(format!("/proc/{}/comm", pid))
            .unwrap_or_else(|_| "kernel".into())
            .trim()
            .to_string();

        // Wyciągamy zużycie RAM z /proc/[pid]/status (linia VmRSS)
        let mut rss_bytes: u64 = 0;
        if let Ok(status_txt) = fs::read_to_string(format!("/proc/{}/status", pid)) {
            for line in status_txt.lines() {
                if let Some(rest) = line.strip_prefix("VmRSS:") {
                    if let Some(kb_str) = rest.split_whitespace().next() {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            rss_bytes = kb * 1024;
                        }
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

/// Czyta statystyki pamięci RAM z /proc/meminfo
fn sys_mem() -> Vec<VosObject> {
    let mut list = Vec::new();
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines().take(5) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(kb) = parts[1].parse::<u64>() {
                    let mut obj = HashMap::new();
                    obj.insert("metric".into(), VosValue::Text(parts[0].trim_end_matches(':').into()));
                    obj.insert("size".into(), VosValue::Bytes(kb * 1024));
                    list.push(obj);
                }
            }
        }
    }
    list
}

/// Czas działania systemu z /proc/uptime
fn sys_uptime() -> Vec<VosObject> {
    let mut obj = HashMap::new();
    let raw = fs::read_to_string("/proc/uptime").unwrap_or_else(|_| "0.0".into());
    let secs = raw.split_whitespace().next().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
    obj.insert("seconds".into(), VosValue::Int(secs as i64));
    vec![obj]
}

/// Listuje pliki i katalogi wraz z ich rozmiarem w bajtach
fn fs_ls(path: &str) -> Result<Vec<VosObject>, String> {
    let entries = fs::read_dir(path).map_err(|e| format!("Błąd odczytu {}: {}", path, e))?;
    let mut list = Vec::new();

    for entry in entries.flatten() {
        let mut obj = HashMap::new();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let meta = entry.metadata();

        let (kind, size) = match meta {
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

/// Odczytuje zawartość pliku linia po linii jako obiekty VOS
fn fs_cat(path: &str) -> Result<Vec<VosObject>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Nie można otworzyć {}: {}", path, e))?;
    let list = content
        .lines()
        .enumerate()
        .map(|(idx, line)| {
            let mut obj = HashMap::new();
            obj.insert("line".into(), VosValue::Int((idx + 1) as i64));
            obj.insert("text".into(), VosValue::Text(line.to_string()));
            obj
        })
        .collect();
    Ok(list)
}
        ("mur", "sync") => run_external("/usr/bin/mur", &["sync"]),
        ("mur", "list") => run_external("/usr/bin/mur", &["list"]),
        ("mur", "install") => {
            let Some(VosValue::Text(pkg)) = args.first() else {
                return Err("mur.install() wymaga nazwy pakietu".into());
            };
            run_external("/usr/bin/mur", &["install", pkg])
        }
