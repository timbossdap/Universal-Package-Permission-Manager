use core::ScanSum;
use core::collectors::flatpak;
use core::collectors::homebrew;
use core::collectors::pacman;
use std::process::Command;

fn is_installed(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn print_profile(profile: &core::AppProf) {
    println!("Profile: {}", profile.app_id);
    if profile.permissions.is_empty() {
        return;
    }
    for perm in &profile.permissions {
        println!("  [{}] {}", perm.cat, perm.desc);
    }
}

fn main() {
    let mut profiles = Vec::new();

    if is_installed("flatpak") {
        match flatpak::collect() {
            Ok(p) => profiles.extend(p),
            Err(e) => eprintln!("Error (flatpak): {}", e),
        }
    } else {
        println!("flatpak is not installed, skipping");
    }

    if is_installed("pacman") {
        match pacman::collect() {
            Ok(p) => profiles.extend(p),
            Err(e) => eprintln!("Error (pacman): {}", e),
        }
    } else {
        println!("pacman is not installed, skipping");
    }

    if is_installed("brew") {
        match homebrew::collect() {
            Ok(p) => profiles.extend(p),
            Err(e) => eprintln!("Error (homebrew): {}", e),
        }
    } else {
        println!("brew is not installed, skipping");
    }

    if profiles.is_empty() {
        println!("No apps found.");
        return;
    }

    for profile in &profiles {
        print_profile(profile);
    }

    let summary = ScanSum::from_profiles(&profiles);
    println!("\n=== Summary ===");
    println!("  Total apps: {}", summary.app_count);
    println!("  Flagged permissions: {}", summary.flagged_count);
}
