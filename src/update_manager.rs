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
                  
                    if output.status.success() {
                        println!("Success: {} installed.", pkg);
                        sender.send("Ok".to_string()).unwrap();
                    } else {
                       
                        eprintln!("Command ran but failed with exit code: {}", output.status);
                        eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
                        sender.send("Err".to_string()).unwrap();
                    }
                }
                Err(e) => {
                  
                    eprintln!("Failed to even launch pkexec: {}", e);
                    sender.send("Err".to_string()).unwrap();
                }
            }
        });
    }
}
