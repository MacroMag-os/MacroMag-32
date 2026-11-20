use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn mount_fs(source: &str, target: &str, fstype: &str) {
    let _ = fs::create_dir_all(target);
    let _ = Command::new("/bin/mount")
        .args(["-t", fstype, source, target])
        .status();
}

fn main() {
    println!("\x1b[1;36m[mm-init]\x1b[0m Starting MacroMag OS (Zero-Telemetry Core)...");

    // 1. Montowanie wirtualnych systemów plików jądra
    mount_fs("proc", "/proc", "proc");
    mount_fs("sysfs", "/sys", "sysfs");
    mount_fs("devtmpfs", "/dev", "devtmpfs");
    let _ = fs::create_dir_all("/dev/pts");
    mount_fs("devpts", "/dev/pts", "devpts");
    mount_fs("tmpfs", "/tmp", "tmpfs");

    // 2. Ustawienie nazwy hosta
    let _ = fs::write("/proc/sys/kernel/hostname", "macromag\n");

    // 3. Podniesienie sieci (loopback + karta eth0 przez DHCP)
    println!("\x1b[1;36m[mm-init]\x1b[0m Initializing network...");
    let _ = Command::new("/bin/ip").args(["link", "set", "lo", "up"]).status();
    let _ = Command::new("/bin/ip").args(["link", "set", "eth0", "up"]).status();
    let _ = Command::new("/bin/udhcpc")
        .args(["-i", "eth0", "-n", "-q"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    // 4. Uruchomienie serwera SSH (Dropbear) w tle
    println!("\x1b[1;36m[mm-init]\x1b[0m Starting Dropbear SSH server on port 22...");
    let _ = fs::create_dir_all("/etc/dropbear");
    let _ = Command::new("/sbin/dropbear")
        .args(["-R", "-B"]) // -R generuje klucze automatycznie, -B pozwala na puste hasła w trybie Live (do zmiany po instalacji)
        .spawn();

    // 5. Główna pętla terminala (uruchamia ekran logowania)
    loop {
        let _ = Command::new("/bin/login").status();
        thread::sleep(Duration::from_millis(500));
    }
}
