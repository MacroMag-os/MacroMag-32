use std::fs;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::process::Command;

fn main() {
    println!("\n==========================================");
    println!("   MacroMag OS v0.1-alpha (x86_64 TTY)    ");
    println!("   Default login: root / pass: macromag   ");
    println!("==========================================\n");

    let users_db = fs::read_to_string("/etc/macromag_users")
        .unwrap_or_else(|_| String::from("root:macromag\n"));

    loop {
        print!("macromag login: ");
        io::stdout().flush().unwrap();
        let mut username = String::new();
        if io::stdin().read_line(&mut username).is_err() {
            continue;
        }
        let username = username.trim();

        print!("Password: ");
        io::stdout().flush().unwrap();
        let mut password = String::new();
        if io::stdin().read_line(&mut password).is_err() {
            continue;
        }
        let password = password.trim();

        let authenticated = users_db.lines().any(|line| {
            if let Some((u, p)) = line.split_once(':') {
                u == username && p == password
            } else {
                false
            }
        });

        if authenticated {
            println!("\nWelcome to MacroMag OS, {}! Launching VOS...\n", username);
            // Uruchomienie powłoki obiektowej VOS
            let err = Command::new("/bin/vos")
                .env("USER", username)
                .env("HOME", if username == "root" { "/root" } else { "/tmp" })
                .env("PATH", "/bin:/sbin:/usr/bin:/usr/sbin")
                .exec();
            eprintln!("Failed to launch VOS: {}", err);
        } else {
            println!("\x1b[1;31mLogin incorrect\x1b[0m\n");
        }
    }
}
