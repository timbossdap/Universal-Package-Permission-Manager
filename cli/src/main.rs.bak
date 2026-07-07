use core::collectors::flatpak::FlatpakCollector;
use core::PermissionCategory;

fn main() {
    let collector = FlatpakCollector::new();
    let result = collector.collect();

    match result {
        Ok(profiles) => {
            if profiles.len() == 0 {
                println!("No flatpak apps found (or flatpak isnt installed)");
                return;
            }

            for profile in &profiles {
                print_profile(profile);
            }
        }
        Err(e) => {
            println!("something went wrong: {}", e);
        }
    }
}

fn print_profile(profile: &core::AppProfile) {
    println!("");
    println!("App: {}", profile.app_id);

    let mut fs_list: Vec<String> = Vec::new();
    let mut hw_list: Vec<String> = Vec::new();
    let mut net_list: Vec<String> = Vec::new();
    let mut desktop_list: Vec<String> = Vec::new();
    let mut sys_list: Vec<String> = Vec::new();

    for p in &profile.permissions {
        if p.category == PermissionCategory::Filesystem {
            fs_list.push(p.description.clone());
        } else if p.category == PermissionCategory::Hardware {
            hw_list.push(p.description.clone());
        } else if p.category == PermissionCategory::Network {
            net_list.push(p.description.clone());
        } else if p.category == PermissionCategory::Desktop {
            desktop_list.push(p.description.clone());
        } else if p.category == PermissionCategory::System {
            sys_list.push(p.description.clone());
        }
    }

    // if x.len() > 0 instead of !x.is_empty(), from a reddit thread
    if fs_list.len() > 0 {
        println!("Filesystem access: {}", fs_list.join(", "));
    }
    if hw_list.len() > 0 {
        println!("Hardware access: {}", hw_list.join(", "));
    }
    if net_list.len() > 0 {
        println!("Network access: {}", net_list.join(", "));
    }
    if desktop_list.len() > 0 {
        println!("Desktop services: {}", desktop_list.join(", "));
    }
    if sys_list.len() > 0 {
        println!("System integration: {}", sys_list.join(", "));
    }
}
