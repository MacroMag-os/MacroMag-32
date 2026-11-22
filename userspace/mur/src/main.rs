use std::collections::HashMap;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

const MUR_ARCHIVE_URL: &str = "https://github.com/Macromag-os/mur";
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
}

fn parse_manifest(pkg_name: &str) -> Option<PkgManifest> {
    let manifest_path = format!("{}/{}/mur.pkg", MUR_PACKAGES_DIR, pkg_name);
    let content = fs::read_to_string(manifest_path).ok()?;
    let mut map = HashMap::new();

    for line in content.lines() {
        if let Some((k, v)) = line.split_once('=') {
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
    })
}

fn download_file(url: &str, dest: &str) -> bool {
    // Najpierw probujemy statycznego curl (najlepsza obsluga HTTPS i przekierowan GitHub)
    if let Ok(status) = Command::new("curl")
        .args(["-fSL", "--connect-timeout", "10", "-o", dest, url])
        .status()
    {
        if status.success() {
            return true;
        }
    }
    // Fallback na wget z BusyBoxa
    Command::new("wget")
        .args(["-q", "-O", dest, url])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args.get(1).map(|s| s.as_str()).unwrap_or("") {
        "sync" => sync_repository(),
        "list" => list_packages(),
        "info" => {
            if let Some(pkg) = args.get(2) {
                info_package(pkg);
            } else {
                eprintln!("[MUR Error]: Podaj nazwe pakietu, np. mur info curl");
            }
        }
        "install" => {
            if let Some(pkg) = args.get(2) {
                install_package(pkg);
            } else {
                eprintln!("[MUR Error]: Podaj nazwe pakietu, np. mur install curl");
            }
        }
        "remove" => {
            if let Some(pkg) = args.get(2) {
                remove_package(pkg);
            } else {
                eprintln!("[MUR Error]: Podaj nazwe pakietu do usuniecia");
            }
        }
        _ => print_help(),
    }
}

fn print_help() {
    println!("MUR (MacroMag User Repository) v0.3.0");
    println!("Uzycie:");
    println!("  mur sync              - pobiera aktualna baze pakietow z GitHuba");
    println!("  mur list              - wyswietla dostepne pakiety i ich wersje");
    println!("  mur info <pakiet>     - pokazuje szczegoly pakietu");
    println!("  mur install <pakiet>  - instaluje narzedzie w /usr/bin");
    println!("  mur remove <pakiet>   - usuwa zainstalowane narzedzie");
}

fn sync_repository() {
    println!("[MUR] Pobieranie bazy pakietow z GitHuba...");
    let _ = fs::create_dir_all(MUR_CACHE_DIR);
    let archive_path = "/tmp/mur-main.tar.gz";

    if !download_file(MUR_ARCHIVE_URL, archive_path) {
        eprintln!("[MUR Error]: Nie udalo sie pobrac bazy (sprawdz siec: ping 1.1.1.1 lub obecnosc curl/ssl_client).");
        return;
    }

    // Czyscimy stary katalog packages i rozpakowujemy nowe archiwum ze zrzutem struktury (--strip-components=1)
    let _ = fs::remove_dir_all(MUR_PACKAGES_DIR);
    let tar_ok = Command::new("/bin/tar")
        .args(["-xzf", archive_path, "-C", MUR_CACHE_DIR, "--strip-components=1"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let _ = fs::remove_file(archive_path);

    if tar_ok {
        println!("[MUR] Baza pakietow zaktualizowana pomyslnie!");
    } else {
        eprintln!("[MUR Error]: Blad rozpakowywania archiwum bazy pakietow.");
    }
}

fn list_packages() {
    println!("Dostepne pakiety w MacroMag User Repository (MUR):");
    let Ok(entries) = fs::read_dir(MUR_PACKAGES_DIR) else {
        println!("  (Brak lokalnej bazy. Uruchom najpierw: mur sync)");
        return;
    };

    let mut count = 0;
    for entry in entries.flatten() {
        if entry.path().is_dir() {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            if let Some(manifest) = parse_manifest(&dir_name) {
                let installed = if fs::metadata(format!("{}/{}", SYSTEM_BIN_DIR, manifest.bin)).is_ok() {
                    " \x1B[1;32m[ZAINSTALOWANO]\x1B[0m"
                } else {
                    ""
                };
                println!("  - \x1B[1;36m{}\x1B[0m ({}){} — {}", manifest.name, manifest.version, installed, manifest.description);
                count += 1;
            }
        }
    }

    if count == 0 {
        println!("  (Brak pakietow. Uruchom najpierw: mur sync)");
    }
}

fn info_package(pkg_name: &str) {
    match parse_manifest(pkg_name) {
        Some(m) => {
            println!("Pakiet:      {}", m.name);
            println!("Wersja:      {}", m.version);
            println!("Plik bin:    {}/{}", SYSTEM_BIN_DIR, m.bin);
            println!("Opis:        {}", m.description);
            println!("Zrodlo URL:  {}", m.url);
        }
        None => eprintln!("[MUR Error]: Nie znaleziono pakietu '{}'.", pkg_name),
    }
}

fn install_package(pkg_name: &str) {
    let Some(manifest) = parse_manifest(pkg_name) else {
        eprintln!("[MUR Error]: Pakiet '{}' nie istnieje w MUR (uruchom 'mur sync' i 'mur list').", pkg_name);
        return;
    };

    if manifest.url.is_empty() {
        eprintln!("[MUR Error]: Manifest mur.pkg dla '{}' nie zawiera pola url=.", pkg_name);
        return;
    }

    let _ = fs::create_dir_all(SYSTEM_BIN_DIR);
    let dst_bin = format!("{}/{}", SYSTEM_BIN_DIR, manifest.bin);

    println!("[MUR] Pobieranie {} v{} z {}...", manifest.name, manifest.version, manifest.url);
    if !download_file(&manifest.url, &dst_bin) {
        eprintln!("[MUR Error]: Pobieranie binarki nie powiodlo sie.");
        return;
    }

    // Opcjonalna weryfikacja sumy kontrolnej SHA256 (jesli podana w mur.pkg)
    if !manifest.sha256.is_empty() {
        if let Ok(out) = Command::new("/bin/sha256sum").arg(&dst_bin).output() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let actual_hash = stdout.split_whitespace().next().unwrap_or("");
            if actual_hash != manifest.sha256 {
                let _ = fs::remove_file(&dst_bin);
                eprintln!("[MUR Security Error]: Niezgodna suma SHA256! Oczekiwano: {}, otrzymano: {}", manifest.sha256, actual_hash);
                return;
            }
        }
    }

    if let Ok(meta) = fs::metadata(&dst_bin) {
        let mut perms = meta.permissions();
        perms.set_mode(0o755);
        let _ = fs::set_permissions(&dst_bin, perms);
    }

    println!("[MUR] Zainstalowano '{}' v{} w {}!", manifest.name, manifest.version, dst_bin);
}

fn remove_package(pkg_name: &str) {
    let bin_name = parse_manifest(pkg_name)
        .map(|m| m.bin)
        .unwrap_or_else(|| pkg_name.to_string());

    let dst_bin = format!("{}/{}", SYSTEM_BIN_DIR, bin_name);
    if fs::remove_file(&dst_bin).is_ok() {
        println!("[MUR] Usunieto pakiet '{}' ({}).", pkg_name, dst_bin);
    } else {
        eprintln!("[MUR Error]: Pakiet '{}' nie jest zainstalowany.", pkg_name);
    }
}
