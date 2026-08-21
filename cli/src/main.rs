use core::collectors::flatpak;
use core::collectors::pacman;

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

    match flatpak::collect() {
        Ok(p) => profiles.extend(p),
        Err(e) => eprintln!("Error (flatpak): {}", e),
    }

    match pacman::collect() {
        Ok(p) => profiles.extend(p),
        Err(e) => eprintln!("Error (pacman): {}", e),
    }

    if profiles.is_empty() {
        println!("No apps found.");
        return;
    }

    for profile in &profiles {
        print_profile(profile);
    }
}
