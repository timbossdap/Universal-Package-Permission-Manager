use core::PermCat;
use core::collectors::flatpak;

fn print_profile(profile: &core::AppProf) {
    println!("Profile: {}", profile.app_id);
    let net_list: Vec<String> = profile
        .permissions
        .iter()
        .filter(|p| p.cat == PermCat::Network)
        .map(|p| p.desc.clone())
        .collect();
    if !net_list.is_empty() {
        println!("Network access: {}", net_list.join(", "));
    }
}

fn main() {
    match flatpak::collect() {
        Ok(profiles) => {
            if !profiles.is_empty() {
                for profile in &profiles {
                    print_profile(profile);
                }
            } else {
                println!("No apps found.");
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
