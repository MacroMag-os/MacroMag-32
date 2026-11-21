use std::env;
use std::fs;
use std::process::Command;

const MUR_REPO_URL: &str = "https://github.com/MacroMag-os/mur";
const MUR_CACHE_DIR: &str = "/var/cache/mur";
const SYSTEM_BIN_DIR: &str = "/usr/bin";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "sync" => sync_repository(),
        "list" => list_packages(),
        "install" => {
            if args.len() < 3 {
                eprintln!("[MUR Error]: Podaj nazwę pakietu, np. mur install <nazwa>");
                return;
            }
            install_package(&args[2]);
        }
        _ => print_help(),
    }
}

fn print_help() {
    println!("MUR (MacroMag User Repository) v0.1.0");
    println!("Użycie:");
    println!("  mur sync              - synchronizuje lokalną bazę z MacroMag-os/mur");
    println!("  mur list              - wyświetla dostępne pakiety w repozytorium");
    println!("  mur install <pakiet>  - kompiluje i instaluje pakiet w /usr/bin");
}

/// Klonuje lub aktualizuje oficjalne repozytorium MacroMag-os/mur
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
        Ok(s) if s.success() => println!("[MUR] Baza pakietów zaktualizowana."),
        _ => eprintln!("[MUR Error]: Brak połączenia sieciowego lub brak narzędzia git."),
    }
}

/// Listuje katalogi pakietów pobrane z repozytorium MUR
fn list_packages() {
    println!("Dostępne pakiety w MUR:");
    match fs::read_dir(MUR_CACHE_DIR) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.starts_with('.') && entry.path().is_dir() {
                    println!("  - {}", name);
                }
            }
        }
        Err(_) => println!("  (Brak lokalnej bazy. Uruchom najpierw: mur sync)"),
    }
}

/// Buduje i instaluje wskazany pakiet z MUR
fn install_package(pkg_name: &str) {
    let pkg_dir = format!("{}/{}", MUR_CACHE_DIR, pkg_name);
    if fs::metadata(&pkg_dir).is_err() {
        eprintln!("[MUR Error]: Pakiet '{}' nie istnieje w lokalnej bazie.", pkg_name);
        return;
    }

    println!("[MUR] Budowanie pakietu '{}'...", pkg_name);
    let build_status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&pkg_dir)
        .status();

    match build_status {
        Ok(s) if s.success() => {
            let src_bin = format!("{}/target/release/{}", pkg_dir, pkg_name);
            let dst_bin = format!("{}/{}", SYSTEM_BIN_DIR, pkg_name);
            let _ = fs::create_dir_all(SYSTEM_BIN_DIR);

            if fs::copy(&src_bin, &dst_bin).is_ok() {
                println!("[MUR] Zainstalowano '{}' w {}", pkg_name, dst_bin);
            } else {
                eprintln!("[MUR Error]: Nie udało się skopiować binarki do {}", dst_bin);
            }
        }
        _ => eprintln!("[MUR Error]: Kompilacja pakietu '{}' nie powiodła się.", pkg_name),
    }
}
