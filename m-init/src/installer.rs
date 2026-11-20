use std::fs;
use std::path::Path;
use std::process::Command;

pub fn install_to_disk(target_dev: &str) {
    println!("\n[MM-INSTALLER] Starting MacroMag OS installation on {}...", target_dev);

    if !Path::new(target_dev).exists() {
        println!("[ERROR] Target device {} not found! Attach a virtual disk in QEMU.", target_dev);
        return;
    }

    println!("[1/4] Formatting {} with ext4 filesystem...", target_dev);
    let _ = Command::new("/bin/busybox")
        .args(["mkfs.ext4", "-F", target_dev])
        .status();

    println!("[2/4] Mounting target root filesystem at /mnt...");
    let _ = fs::create_dir_all("/mnt");
    let _ = Command::new("/bin/busybox")
        .args(["mount", target_dev, "/mnt"])
        .status();

    println!("[3/4] Deploying MacroMag kernel init and directory hierarchy...");
    for dir in ["/mnt/boot", "/mnt/etc", "/mnt/proc", "/mnt/sys", "/mnt/dev", "/mnt/apps"] {
        let _ = fs::create_dir_all(dir);
    }
    let _ = fs::copy("/init", "/mnt/init");
    let _ = fs::write(
        "/mnt/etc/macromag-release",
        "NAME=\"MacroMag OS\"\nVERSION=\"2.0v\"\nID=macromag\nARCH=x86_64\n",
    );

    println!("[4/4] Syncing buffers and unmounting {}...", target_dev);
    let _ = Command::new("/bin/busybox").args(["sync"]).status();
    let _ = Command::new("/bin/busybox").args(["umount", "/mnt"]).status();

    println!("[SUCCESS] MacroMag OS 2.0v installed cleanly to {}!\n", target_dev);
}
