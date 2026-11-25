use std::collections::HashMap;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

const MUR_ARCHIVE_URL: &str = "https://github.com";
const MUR_CACHE_DIR: &str = "/var/cache/mur";
const MUR_PACKAGES_DIR: &str = "/var/cache/mur/packages";
const SYSTEM_BIN_DIR: &str = "/usr/bin";

struct PkgManifest {
    name: String,
    version: String,
    description: String,
    url: String,
    bin: String,
    sha256: String,
    archive: String,
}

fn parse_manifest(pkg_name: &str) -> Option<PkgManifest> {
    let manifest_path = format!("{}/{}/mur.pkg", MUR_PACKAGES_DIR, pkg_name);
    let content = fs::read_to_string(manifest_path).ok()?;
    let mut map = HashMap::new();
    for line in content.lines() {
        if let Some((k, v)) = line.split_once("=") {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Some(PkgManifest {
        name: map.get("name").cloned().unwrap_or_else(|| pkg_name.to_string()),
        version: map.get("version").cloned().unwrap_or_else(|| "unknown".to_string()),
        description: map.get("description").cloned().unwrap_or_else(|| "Brak opisu".to_string()),
        url: map.get("url").cloned().unwrap_or_default(),
        bin: map.get("bin").cloned().unwrap_or_else(|| pkg_name.to_string()),
        sha256: map.get("sha256").cloned().unwrap_or_default(),
        archive: map.get("archive").cloned().unwrap_or_default(),
    })
}

fn download_file(url: &str, dest: &str) -> bool {
    if let Ok(status) = Command::new("curl").args(["-fSL", "--connect-timeout", "10", "-o", dest, url]).status() {
        if status.success() { return true; }
    }
    Command::new("wget").args(["-q", "-O", dest, url]).status().map(|s| s.success()).unwrap_or(false)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { print_help(); return; }
    match args.get(1).map(|s| s.as_str()).unwrap_or("") {
        "sync" => sync_repository(),
        "list" => list_packages(),
        "info" => if let Some(pkg) = args.get(2) { info_package(pkg); } else { eprintln!("[MUR Error]: Podaj nazwe pakietu"); },
        "install" => if let Some(pkg) = args.get(2) { install_package(pkg); } else { eprintln!("[MUR Error]: Podaj nazwe pakietu"); },
        "sys-check" => sys_check(),
        "sys-upgrade" => sys_upgrade(),
        "remove" => if let Some(pkg) = args.get(2) { remove_package(pkg); } else { eprintln!("[MUR Error]: Podaj nazwe pakietu"); },
        _ => print_help(),
    }
}

fn print_help() {
    println!("MUR (MacroMag User Repository) v0.3.0\nUzycie: mur sync | list | info <pkg> | install <pkg> | remove <pkg>");
}

fn sync_repository() {
    println!("[MUR] Downloading bazy pakietow z GitHuba...");
    let _ = fs::create_dir_all(MUR_CACHE_DIR);
    let archive_path = "/tmp/mur-main.tar.gz";
    if !download_file(MUR_ARCHIVE_URL, archive_path) {
        eprintln!("[MUR Error]: Blad pobierania bazy.");
        return;
    }
    let _ = fs::remove_dir_all(MUR_PACKAGES_DIR);
    let tar_ok = Command::new("/bin/tar").args(["-xzf", archive_path, "-C", MUR_CACHE_DIR, "--strip-components=1"]).status().map(|s| s.success()).unwrap_or(false);
    let _ = fs::remove_file(archive_path);
    if tar_ok { println!("[MUR] Baza pakietow zaktualizowana!"); } else { eprintln!("[MUR Error]: Blad rozpakowywania."); }
}

fn list_packages() {
    println!("Dostepne pakiety w MUR:");
    let Ok(entries) = fs::read_dir(MUR_PACKAGES_DIR) else { println!("  (Uruchom najpierw: mur sync)"); return; };
    for entry in entries.flatten() {
        if entry.path().is_dir() {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if let Some(m) = parse_manifest(&dir_name) {
                let inst = if fs::metadata(format!("{}/{}", SYSTEM_BIN_DIR, m.bin)).is_ok() { " [ZAINSTALOWANO]" } else { "" };
                println!("  - {} ({}){} — {}", m.name, m.version, inst, m.description);
            }
        }
    }
}

fn info_package(pkg_name: &str) {
    match parse_manifest(pkg_name) {
        Some(m) => println!("Pakiet: {} ({})\nBin: {}/{}\nOpis: {}\nURL: {}", m.name, m.version, SYSTEM_BIN_DIR, m.bin, m.description, m.url),
        None => eprintln!("[MUR Error]: Nie znaleziono pakietu {}.", pkg_name),
    }
}

fn install_package(pkg_name: &str) {
    let Some(m) = parse_manifest(pkg_name) else { eprintln!("[MUR Error]: Pakiet {} nie istnieje.", pkg_name); return; };
    let _ = fs::create_dir_all(SYSTEM_BIN_DIR);
    let dst_bin = format!("{}/{}", SYSTEM_BIN_DIR, m.bin);
    println!("[MUR] Downloading {} v{}...", m.name, m.version);
    if !download_file(&m.url, &dst_bin) { eprintln!("[MUR Error]: Downloading nie powiodlo sie."); return; }
    if !m.sha256.is_empty() {
        if let Ok(out) = Command::new("/bin/sha256sum").arg(&dst_bin).output() {
            let actual = String::from_utf8_lossy(&out.stdout).split_whitespace().next().unwrap_or("").to_string();
            if actual != m.sha256 { let _ = fs::remove_file(&dst_bin); eprintln!("[MUR Security Error]: Zly SHA256!"); return; }
        }
    }
    if let Ok(meta) = fs::metadata(&dst_bin) { let mut p = meta.permissions(); p.set_mode(0o755); let _ = fs::set_permissions(&dst_bin, p); }
    println!("[MUR] Installed {} w {}!", m.name, dst_bin);
}

fn remove_package(pkg_name: &str) {
    let bin = parse_manifest(pkg_name).map(|m| m.bin).unwrap_or_else(|| pkg_name.to_string());
    if fs::remove_file(format!("{}/{}", SYSTEM_BIN_DIR, bin)).is_ok() { println!("[MUR] Usunieto {}.", pkg_name); } else { eprintln!("[MUR Error]: Nie znaleziono."); }
}


const OS_RELEASE_API: &str = "https://github.com";
const VERSION_FILE: &str = "/etc/macromag_version";

fn get_local_os_version() -> String {
    fs::read_to_string(VERSION_FILE).unwrap_or_else(|_| "v0.1.0".into()).trim().to_string()
}

fn fetch_latest_release_info() -> Option<(String, String, String)> {
    let out = Command::new("curl")
        .args(["-fsSL", "-H", "User-Agent: MacroMag-MUR", OS_RELEASE_API])
        .output().ok()?;
    if !out.status.success() { return None; }
    let json = String::from_utf8_lossy(&out.stdout);
    let tag = json.split("\"tag_name\":\"").nth(1)?.split("\"").next()?.to_string();
    let mut vos_url = String::new();
    let mut mur_url = String::new();
    for part in json.split("\"browser_download_url\":\"") {
        if let Some(u) = part.split("\"").next() {
            if u.ends_with("/vos") { vos_url = u.to_string(); }
            if u.ends_with("/mur") { mur_url = u.to_string(); }
        }
    }
    Some((tag, vos_url, mur_url))
}

fn sys_check() {
    let local = get_local_os_version();
    println!("[MUR OTA] Lokalna wersja MacroMag OS: {}", local);
    match fetch_latest_release_info() {
        Some((latest, vos_u, _)) => {
            println!("[MUR OTA] Najnowsze wydanie na GitHub: {}", latest);
            if local != latest {
                println!("\x1B[1;33m[DOSTEPNA AKTUALIZACJA]\x1B[0m Wpisz: mur sys-upgrade");
                if vos_u.is_empty() { println!("(Uwaga: to wydanie zawiera tylko ISO — pobierz je z GitHub Releases)"); }
            } else {
                println!("\x1B[1;32m[OK]\x1B[0m Masz najnowsza wersje systemu!");
            }
        }
        None => eprintln!("[MUR Error]: Nie udalo sie polaczyc z GitHub Releases API."),
    }
}

fn sys_upgrade() {
    let Some((latest, vos_url, mur_url)) = fetch_latest_release_info() else {
        eprintln!("[MUR Error]: Brak polaczenia z GitHub Releases.");
        return;
    };
    if vos_url.is_empty() || mur_url.is_empty() {
        eprintln!("[MUR OTA]: Wydanie {} nie zawiera samodzielnych binarek vos/mur (wymaga nowego ISO).", latest);
        return;
    }
    println!("[MUR OTA] Aktualizacja komponentow bazowych do {}...", latest);
    if download_file(&vos_url, "/bin/vos.new") && download_file(&mur_url, "/usr/bin/mur.new") {
        let _ = fs::set_permissions("/bin/vos.new", fs::Permissions::from_mode(0o755));
        let _ = fs::set_permissions("/usr/bin/mur.new", fs::Permissions::from_mode(0o755));
        let _ = fs::rename("/bin/vos.new", "/bin/vos");
        let _ = fs::copy("/bin/vos", "/init");
        let _ = fs::rename("/usr/bin/mur.new", "/usr/bin/mur");
        let _ = fs::write(VERSION_FILE, format!("{}\n", latest));
        println!("\x1B[1;32m[SUKCES]\x1B[0m Zaktualizowano system do {}! Wpisz sys.reboot().", latest);
    } else {
        eprintln!("[MUR Error]: Nie udalo sie pobrac plikow aktualizacji.");
    }
}
