use std::process::Command;
use std::sync::mpsc;
use std::thread;

pub struct UpdateManager;

impl UpdateManager {
    pub fn check_updates(sender: mpsc::Sender<String>) {
        thread::spawn(move || {
            let output = Command::new("checkupdates").output();
            let result = match output {
                Ok(res) => String::from_utf8_lossy(&res.stdout).to_string(),
                Err(_) => "Error".to_string(),
            };
            let _ = sender.send(result);
        });
    }

    pub fn install_package(pkg: String, sender: mpsc::Sender<String>) {
        println!("Thread started for: {}", pkg);
        thread::spawn(move || {
            match Command::new("pkexec")
                .args(["pacman", "-y", "-S", &pkg, "--noconfirm"])
                .output()
            {
                Ok(output) => {
                    println!("Command executed for {}", pkg);
                    println!("Status: {}", output.status);
                    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
                    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
                    sender.send(("Ok").to_string());
                }
                Err(e) => {
                    eprintln!("Command failed to execute for {}: {}", pkg, e);
                    sender.send(("Err").to_string());
                }
            }
            
        });
    }
}
