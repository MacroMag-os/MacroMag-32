use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

const MUR_REPO_URL: &str = "https://github.com/Macromag-os/mur";
const MUR_CACHE_DIR: &str = "/var/cache/mur";
const MUR_PACKAGES_DIR: &str = "/var/cache/mur/packages";
const SYSTEM_BIN_DIR: &str = "/usr/bin";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args.get(1).map(|s| s.as_str()).unwrap_or("") {
        "sync" => sync_repository(),
        "list" => list_packages(),
        "install" => {
            if let Some(pkg) = args.get(2) {
                install_package(pkg);
            } else {
                eprintln!("[MUR Error]: Podaj nazwe pakietu, np. mur install htop");
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
    println!("MUR (MacroMag User Repository) v0.2.0");
    println!("Polaczono z: {}", MUR_REPO_URL);
    println!("Uzycie:");
    println!("  mur sync              - pobiera liste pakietow z MacroMag-os/mur");
    println!("  mur list              - wyswietla dostepne narzedzia w MUR");
    println!("  mur install <pakiet>  - instaluje narzedzie w /usr/bin");
    println!("  mur remove <pakiet>   - usuwa zainstalowane narzedzie");
}

fn sync_repository() {
    println!("[MUR] Synchronizacja z {}...", MUR_REPO_URL);
    let _ = fs::create_dir_all(MUR_CACHE_DIR);

    let status = if fs::metadata(format!("{}/.git", MUR_CACHE_DIR)).is_ok() {
        Command::new("git")
            .args(["-C", MUR_CACHE_DIR, "pull", "--quiet"])
            .status()
    } else {
        Command::new("git")
            .args(["clone", "--depth=1", MUR_REPO_URL, MUR_CACHE_DIR])
            .status()
    };

    match status {
        Ok(s) if s.success() => println!("[MUR] Baza pakietow zaktualizowana pomyslnie."),
        _ => eprintln!("[MUR Error]: Blad polaczenia z GitHubem (sprawdz siec: ping 1.1.1.1)."),
    }
}

fn list_packages() {
    println!("Dostepne pakiety w MacroMag User Repository (MUR):");
    match fs::read_dir(MUR_PACKAGES_DIR) {
        Ok(entries) => {
            let mut count = 0;
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let installed = if fs::metadata(format!("{}/{}", SYSTEM_BIN_DIR, name)).is_ok() {
                        " [ZAINSTALOWANO]"
                    } else {
                        ""
                    };
                    println!("  - {}{}", name, installed);
                    count += 1;
                }
            }
            if count == 0 {
                println!("  (Brak pakietow. Uruchom najpierw: mur sync)");
            }
        }
        Err(_) => println!("  (Brak lokalnej bazy. Uruchom najpierw: mur sync)"),
    }
}

fn install_package(pkg_name: &str) {
    let pkg_dir = format!("{}/{}", MUR_PACKAGES_DIR, pkg_name);
    if fs::metadata(&pkg_dir).is_err() {
        eprintln!("[MUR Error]: Pakiet '{}' nie istnieje w MUR (uruchom 'mur sync' i 'mur list').", pkg_name);
        return;
    }

    let _ = fs::create_dir_all(SYSTEM_BIN_DIR);
    let dst_bin = format!("{}/{}", SYSTEM_BIN_DIR, pkg_name);

    // 1. Jesli istnieje manifest mur.pkg z adresem URL statycznej binarki -> pobieramy przez wget
    let manifest_path = format!("{}/mur.pkg", pkg_dir);
    if let Ok(manifest) = fs::read_to_string(&manifest_path) {
        for line in manifest.lines() {
            if let Some(url) = line.strip_prefix("url=") {
                println!("[MUR] Pobieranie '{}' z {}...", pkg_name, url.trim());
                let dl = Command::new("wget")
                    .args(["-q", "-O", &dst_bin, url.trim()])
                    .status();

                if let Ok(s) = dl {
                    if s.success() {
                        if let Ok(meta) = fs::metadata(&dst_bin) {
                            let mut perms = meta.permissions();
                            perms.set_mode(0o755);
                            let _ = fs::set_permissions(&dst_bin, perms);
                        }
                        println!("[MUR] Zainstalowano '{}' w {}! Mozesz juz go uruchomic.", pkg_name, dst_bin);
                        return;
                    }
                }
                eprintln!("[MUR Error]: Pobieranie binarki nie powiodlo sie.");
                return;
            }
        }
    }

    // 2. Jesli pakiet zawiera gotowa binarke bezposrednio w katalogu pakietu
    let prebuilt_bin = format!("{}/{}", pkg_dir, pkg_name);
    if fs::metadata(&prebuilt_bin).is_ok() {
        if fs::copy(&prebuilt_bin, &dst_bin).is_ok() {
            if let Ok(meta) = fs::metadata(&dst_bin) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = fs::set_permissions(&dst_bin, perms);
            }
            println!("[MUR] Zainstalowano '{}' z lokalnego repozytorium!", pkg_name);
            return;
        }
    }

    eprintln!("[MUR Error]: Pakiet '{}' nie zawiera pliku mur.pkg z polem url= ani gotowej binarki.", pkg_name);
}

fn remove_package(pkg_name: &str) {
    let dst_bin = format!("{}/{}", SYSTEM_BIN_DIR, pkg_name);
    if fs::remove_file(&dst_bin).is_ok() {
        println!("[MUR] Usunieto pakiet '{}'.", pkg_name);
    } else {
        eprintln!("[MUR Error]: Pakiet '{}' nie jest zainstalowany.", pkg_name);
    }
}
