
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

type VosObj = BTreeMap<String, String>;

fn mount_virtual_fs() {
    for dir in ["/proc", "/sys", "/dev", "/mnt", "/usr/bin", "/etc"] {
        let _ = fs::create_dir_all(dir);
    }
    let _ = Command::new("/bin/busybox").args(["mount", "-t", "proc", "none", "/proc"]).status();
    let _ = Command::new("/bin/busybox").args(["mount", "-t", "sysfs", "none", "/sys"]).status();
    let _ = Command::new("/bin/busybox").args(["mount", "-t", "devtmpfs", "none", "/dev"]).status();
    let _ = Command::new("/bin/busybox").args(["mdev", "-s"]).status();

    // Automatyczne ladowanie sterownikow PCI/USB/NVMe/Ethernet na prawdziwym sprzecie
    let _ = Command::new("/bin/busybox")
        .args(["sh", "-c", "find /sys/devices -name modalias -exec cat {} + | sort -u | xargs -r /bin/busybox modprobe -a -q"])
        .status();
    let _ = Command::new("/bin/busybox").args(["mdev", "-s"]).status();
}

fn init_network() {
    let _ = fs::write("/etc/resolv.conf", "nameserver 1.1.1.1\nnameserver 8.8.8.8\n");
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let iface = entry.file_name().to_string_lossy().to_string();
            if iface == "lo" {
                continue;
            }
            let _ = Command::new("/bin/busybox").args(["ip", "link", "set", &iface, "up"]).status();
            let _ = Command::new("/bin/busybox")
                .args(["udhcpc", "-i", &iface, "-n", "-q", "-s", "/usr/share/udhcpc/default.script"])
                .status();
        }
    }
}

fn print_banner() {
    println!("\x1b[2J\x1b[H");
    println!("==================================================================");
    println!("  MM > MacroMag OS 2.0v (Native Bare-Metal Linux Core)");
    println!("  Engine: Virtualized Object Shell (VOS v2) + MUR Package Manager");
    println!("==================================================================");
    println!("  Wpisz 'help' aby zobaczyc obiekty VOS, MUR oraz instalator.\n");
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

fn mur_base_url() -> String {
    ["https://", "://githubusercontent.com", "/MacroMag-os/MUR/mur-tools/"].concat()
}

fn get_mur_list() -> Vec<VosObj> {
    let mut list = Vec::new();
    let url = format!("{}index.json", mur_base_url());
    let Ok(resp) = ureq::get(&url).call() else {
        println!("[MUR] Brak polaczenia z repozytorium MacroMag-os/MUR.");
        return list;
    };
    let Ok(body) = resp.into_string() else {
        return list;
    };

    let mut current = VosObj::new();
    for line in body.lines() {
        let l = line.trim().trim_end_matches(',');
        if l == "{" {
            current = VosObj::new();
        } else if l == "}" {
            if let Some(name) = current.get("name") {
                let installed = if Path::new(&format!("/usr/bin/{}", name)).exists() { "yes" } else { "no" };
                current.insert("installed".into(), installed.into());
                list.push(current.clone());
            }
        } else if let Some((k, v)) = l.split_once(':') {
            let key = k.trim().trim_matches('"').to_string();
            let val = v.trim().trim_matches('"').to_string();
            if !key.is_empty() && key != "packages" {
                current.insert(key, val);
            }
        }
    }
    list
}

fn mur_install(pkg_name: &str) {
    let pkg = pkg_name.trim();
    if pkg.is_empty() {
        println!("[MUR] Uzycie: mur install <nazwa_pakietu> (np. mur install fastfetch)");
        return;
    }

    let url = format!("{}{}", mur_base_url(), pkg);
    let dest = format!("/usr/bin/{}", pkg);

    println!("[MUR] Pobieranie '{}' z galezi mur-tools...", pkg);
    match ureq::get(&url).call() {
        Ok(resp) => {
            let mut bytes = Vec::new();
            if resp.into_reader().read_to_end(&mut bytes).is_ok() && !bytes.is_empty() {
                if let Ok(mut file) = File::create(&dest) {
                    if file.write_all(&bytes).is_ok() {
                        let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(0o755));
                        println!("[MUR] Zainstalowano '{}' w /usr/bin/{}! Wpisz '{}', aby uruchomic.", pkg, pkg, pkg);
                        return;
                    }
                }
            }
            println!("[MUR] Blad zapisu pakietu '{}'.", pkg);
        }
        Err(_) => {
            println!("[MUR] Nie znaleziono pakietu '{}' w repozytorium MUR (sprawdz 'mur.list()').", pkg);
        }
    }
}

fn run_installer() {
    println!("\n[MM-INSTALLER] Szukam dysku docelowego (/dev/sda lub /dev/nvme0n1)...");
    let target_dev = if Path::new("/dev/nvme0n1").exists() {
        "/dev/nvme0n1"
    } else if Path::new("/dev/sda").exists() {
        "/dev/sda"
    } else {
        println!("[BLAD] Nie wykryto dysku /dev/sda ani /dev/nvme0n1!");
        return;
    };

    println!("[1/4] Formatowanie {} jako FAT32 (ESP/Boot)...", target_dev);
    let _ = Command::new("/bin/busybox").args(["mkfs.vfat", "-F", "32", target_dev]).status();

    println!("[2/4] Montowanie {} w /mnt...", target_dev);
    let _ = Command::new("/bin/busybox").args(["mount", target_dev, "/mnt"]).status();

    println!("[3/4] Kopiowanie jadra, initramfs oraz bootloadera Limine (UEFI + BIOS)...");
    let _ = fs::create_dir_all("/mnt/boot/limine");
    let _ = fs::create_dir_all("/mnt/EFI/BOOT");
    let _ = fs::copy("/boot/vmlinuz", "/mnt/boot/vmlinuz");
    let _ = fs::copy("/boot/initramfs.img", "/mnt/boot/initramfs.img");
    let _ = fs::copy("/boot/limine/limine-bios.sys", "/mnt/boot/limine/limine-bios.sys");
    let _ = fs::copy("/boot/limine/BOOTX64.EFI", "/mnt/EFI/BOOT/BOOTX64.EFI");

    let limine_cfg = "timeout: 3\n\n/MacroMag OS (Installed on Disk)\n    protocol: linux\n    kernel_path: boot():/boot/vmlinuz\n    module_path: boot():/boot/initramfs.img\n    cmdline: quiet loglevel=3 console=tty0\n";
    let _ = fs::write("/mnt/boot/limine/limine.conf", limine_cfg);

    println!("[4/4] Zapisywanie sektora rozruchowego Limine...");
    let _ = Command::new("/bin/busybox").args(["sync"]).status();
    let _ = Command::new("/bin/busybox").args(["umount", "/mnt"]).status();
    let _ = Command::new("/bin/limine").args(["bios-install", target_dev]).status();

    println!("[SUKCES] MacroMag OS zainstalowany na {}! Mozesz wyjac pendrive i uruchomic PC.\n", target_dev);
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
Używaj kodu z rozwagą.
items.retain(|obj| obj.get(k).map(|x| x.contains(v)).unwrap_or(false));
}
} else if s == "to.json()" {
as_json = true;
}
}
if as_json {
println!("[");
for (i, obj) in items.iter().enumerate() {
let fields: Vec = obj.iter().map(|(k, v)| format!(""{}": "{}"", k, v)).collect();
let comma = if i + 1 < items.len() { "," } else { "" };
println!("  {{ {} }}{}", fields.join(", "), comma);
}
println!("]");
} else {
for obj in items {
let fields: Vec = obj.iter().map(|(k, v)| format!("{}: "{}"", k, v)).collect();
println!("  {{ {} }}", fields.join(", "));
}
}
}
fn main() {
mount_virtual_fs();
init_network();
print_banner();
let stdin = io::stdin();
loop {
print!("MM-VOS > ");
let _ = io::stdout().flush();
let mut input = String::new();
if stdin.read_line(&mut input).is_err() {
continue;
}
let raw_line = input.trim();
if raw_line.is_empty() {
continue;
}
let line = raw_line.strip_prefix("sudo ").unwrap_or(raw_line).trim();
if let Some(pkg) = line.strip_prefix("mur install ") {
mur_install(pkg);
continue;
}
let parts: Vec<&str> = line.split('|').collect();
let source = parts[0].trim();
let pipes = &parts[1..];
match source {
"help" => {
println!("Obiekty zrodlowe VOS (laczenie: '| where ...' oraz '| to.json()'):");
println!("  sys.info()      - Informacje o systemie, jadrze i RAM");
println!("  proc.list()     - Lista aktywnych procesow w jadrze");
println!("  disk.list()     - Wykryte dyski twarde i urzadzenia blokowe");
println!("  files.list()    - Obiekty w biezacym katalogu");
println!("  mur.list()      - Lista 50 pakietow z repozytorium MUR na GitHubie");
println!("  sys.install()   - Instaluje MacroMag OS na dysk twardy (/dev/sda lub nvme)");
println!("Komendy MUR i systemowe:");
println!("  mur install   - Instaluje narzedzie z galezi mur-tools (np. fastfetch)");
println!("  clear / reboot       - Czyszczenie ekranu / Restart systemu");
}
"sys.info()" => render_objects(get_sys_info(), pipes),
"proc.list()" => render_objects(get_proc_list(), pipes),
"disk.list()" => render_objects(get_disk_list(), pipes),
"files.list()" => render_objects(get_files_list(), pipes),
"mur.list()" => render_objects(get_mur_list(), pipes),
"sys.install()" => run_installer(),
"clear" => print_banner(),
"reboot" => {
let _ = Command::new("/bin/busybox").args(["reboot", "-f"]).status();
}
_ => {
let mut cmd_parts = source.split_whitespace();
if let Some(bin) = cmd_parts.next() {
let bin_path = format!("/usr/bin/{}", bin);
if Path::new(&bin_path).exists() {
let args: Vec<&str> = cmd_parts.collect();
let _ = Command::new(&bin_path).args(&args).status();
} else {
println!("[BLAD] Nieznana komenda '{}'. Wpisz 'help' lub 'mur.list()'.", bin);
}
}
}
}
}
}

---

### Plik 2: `build_iso.sh` (w repozytorium `MacroMag-32`)
Zapisuje jądro, `initramfs` i binarkę `limine` również wewnątrz samego obrazu systemu (`/boot`), dzięki czemu polecenie `sys.install()` na prawdziwym PC przenosi pełny system na dysk twardy.

```bash
#!/bin/bash
set -e

echo "=== 0. Instalacja narzedzi budujacych ==="
sudo apt-get update
sudo apt-get install -y xorriso cpio curl git build-essential kmod

echo "=== 1. Kompilacja statycznej binarki mm-init ==="
rustup target add x86_64-unknown-linux-musl
cd mm-init
cargo build --release --target x86_64-unknown-linux-musl
cd ..

echo "=== 2. Pobieranie jadra Linux oraz bootloadera Limine ==="
rm -rf build_tmp
mkdir -p build_tmp/rootfs/{bin,sbin,etc,proc,sys,dev,usr/bin,lib/modules,boot/limine}

KVER=$(uname -r)
sudo cp /boot/vmlinuz-$KVER build_tmp/vmlinuz
sudo chmod 644 build_tmp/vmlinuz

git clone https://github.com/limine-bootloader/limine.git --branch=v8.x-binary --depth=1 build_tmp/limine
make -C build_tmp/limine

echo "=== 3. Budowanie struktury systemu (rootfs) ==="
cp mm-init/target/x86_64-unknown-linux-musl/release/mm-init build_tmp/rootfs/init
chmod +x build_tmp/rootfs/init

if [ -f etc/os-release ]; then
    cp etc/os-release build_tmp/rootfs/etc/os-release
fi

# Kopiowanie plikow rozruchowych do /boot wewnatrz systemu (dla sys.install)
cp build_tmp/vmlinuz build_tmp/rootfs/boot/vmlinuz
cp build_tmp/limine/limine-bios.sys build_tmp/rootfs/boot/limine/
cp build_tmp/limine/BOOTX64.EFI build_tmp/rootfs/boot/limine/
cp build_tmp/limine/limine build_tmp/rootfs/bin/limine
chmod +x build_tmp/rootfs/bin/limine

# Statyczny BusyBox (narzedzia + mkfs.vfat + modprobe + udhcpc)
curl -L https://busybox.net/downloads/binaries/1.35.0-x86_64-linux-musl/busybox -o build_tmp/rootfs/bin/busybox
chmod +x build_tmp/rootfs/bin/busybox
for cmd in sh mount umount mdev modprobe insmod udhcpc ip ifconfig ping ls cat mkdir rm cp mv clear mkfs.vfat sync reboot find sort xargs; do
    ln -s busybox build_tmp/rootfs/bin/$cmd
done

mkdir -p build_tmp/rootfs/usr/share/udhcpc
cat << 'EOF' > build_tmp/rootfs/usr/share/udhcpc/default.script
#!/bin/sh
[ "$1" = "bound" ] || [ "$1" = "renew" ] && {
    ip addr add "$ip/$mask" dev "$interface"
    [ -n "$router" ] && ip route add default via "$router" dev "$interface"
    echo "nameserver 1.1.1.1" > /etc/resolv.conf
    echo "nameserver 8.8.8.8" >> /etc/resolv.conf
}
EOF
chmod +x build_tmp/rootfs/usr/share/udhcpc/default.script

# Sterowniki sprzetowe dla fizycznego PC
sudo cp -r /lib/modules/$KVER build_tmp/rootfs/lib/modules/
sudo depmod -b build_tmp/rootfs $KVER

# Pakowanie initramfs.img (najpierw bez samego initramfs w /boot, potem kopiujemy go do /boot)
cd build_tmp/rootfs
find . | cpio -o -H newc | gzip > ../initramfs.img
cp ../initramfs.img boot/initramfs.img
find . | cpio -o -H newc | gzip > ../initramfs.img
cd ../..

echo "=== 4. Tworzenie hybrydowego obrazu MacroMag-32.iso ==="
mkdir -p build_tmp/iso_root/boot/limine
mkdir -p build_tmp/iso_root/EFI/BOOT

cp build_tmp/vmlinuz build_tmp/iso_root/boot/
cp build_tmp/initramfs.img build_tmp/iso_root/boot/

cat << 'EOF' > build_tmp/iso_root/boot/limine/limine.conf
timeout: 3

/MacroMag OS (Native Bare-Metal)
    protocol: linux
    kernel_path: boot():/boot/vmlinuz
    module_path: boot():/boot/initramfs.img
    cmdline: quiet loglevel=3 console=tty0
EOF

cp build_tmp/limine/limine-bios.sys build_tmp/limine/limine-bios-cd.bin build_tmp/limine/limine-uefi-cd.bin build_tmp/iso_root/boot/limine/
cp build_tmp/limine/BOOTX64.EFI build_tmp/iso_root/EFI/BOOT/

xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --efi-boot boot/limine/limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image --protective-msdos-label \
    build_tmp/iso_root -o MacroMag-32.iso

./build_tmp/limine/limine bios-install MacroMag-32.iso

echo "=== GOTOWE! Wygenerowano MacroMag-32.iso ==="
