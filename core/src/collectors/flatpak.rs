use std::collections::HashMap;
use std::process::Command;

use crate::AppProfile;
use crate::Permission;
use crate::PermissionCategory;

pub struct FlatpakCollector;

impl FlatpakCollector {
    pub fn new() -> FlatpakCollector {
        FlatpakCollector
    }

    pub fn collect(&self) -> Result<Vec<AppProfile>, String> {
        let app_ids = list_app_ids()?;
        let mut profiles = Vec::new();

        for id in app_ids {
            let result = build_profile(&id);
            match result {
                Ok(profile) => {
                    profiles.push(profile);
                }
                Err(e) => {
                    println!("couldn't get permissions for {}: {}", id, e);
                }
            }
        }

        Ok(profiles)
    }
}

fn list_app_ids() -> Result<Vec<String>, String> {
    let output = Command::new("flatpak")
        .arg("list")
        .arg("--app")
        .arg("--columns=application")
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Err("flatpak isnt installed i think? or not on path".to_string()),
    };

    if !output.status.success() {
        return Err("flatpak list command failed".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let mut ids = Vec::new();
    for line in text.lines() {
        let clean = line.trim();
        if clean != "" {
            ids.push(clean.to_string());
        }
    }

    Ok(ids)
}

fn build_profile(app_id: &str) -> Result<AppProfile, String> {
    let output = Command::new("flatpak")
        .arg("info")
        .arg("--show-permissions")
        .arg(app_id)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Err("couldnt run flatpak".to_string()),
    };

    if !output.status.success() {
        return Err("flatpak info command failed".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();

    let mut current_section = String::new();
    let mut context_map: HashMap<String, String> = HashMap::new();

    for line in text.lines() {
        let line = line.trim();

        if line.len() == 0 {
            continue;
        }

        if line.starts_with("[") {
            current_section = line.replace("[", "").replace("]", "");
            continue;
        }

        if current_section == "Context" {
            let parts: Vec<&str> = line.split("=").collect();
            if parts.len() == 2 {
                context_map.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
    }

    let mut permissions: Vec<Permission> = Vec::new();

    if let Some(shared) = context_map.get("shared") {
        if shared.contains("network") {
            permissions.push(Permission {
                category: PermissionCategory::Network,
                description: "Internet and local network access".to_string(),
                source_mechanism: "flatpak".to_string(),
                raw: shared.clone(),
            });
        }
    }

    if let Some(sockets) = context_map.get("sockets") {
        if sockets.contains("x11") {
            permissions.push(Permission {
                category: PermissionCategory::Desktop,
                description: "Can see and control other windows (X11)".to_string(),
                source_mechanism: "flatpak".to_string(),
                raw: sockets.clone(),
            });
        }
        if sockets.contains("wayland") {
            permissions.push(Permission {
                category: PermissionCategory::Desktop,
                description: "Wayland display access".to_string(),
                source_mechanism: "flatpak".to_string(),
                raw: sockets.clone(),
            });
        }
        // https://stackoverflow.com/questions/tagged/pulseaudio
        if sockets.contains("pulseaudio") {
            permissions.push(Permission {
                category: PermissionCategory::Hardware,
                description: "Audio playback/recording access".to_string(),
                source_mechanism: "flatpak".to_string(),
                raw: sockets.clone(),
            });
        }
    }

    if let Some(devices) = context_map.get("devices") {
        if devices.contains("dri") {
            permissions.push(Permission {
                category: PermissionCategory::Hardware,
                description: "GPU acceleration".to_string(),
                source_mechanism: "flatpak".to_string(),
                raw: devices.clone(),
            });
        }
        if devices.contains("all") {
            permissions.push(Permission {
                category: PermissionCategory::Hardware,
                description: "Access to all hardware devices (camera, mic, etc)".to_string(),
                source_mechanism: "flatpak".to_string(),
                raw: devices.clone(),
            });
        }
    }

    if let Some(filesystems) = context_map.get("filesystems") {
        let fs_list: Vec<&str> = filesystems.split(";").collect();
        for fs in fs_list {
            let fs = fs.trim();
            if fs == "" {
                continue;
            }

            let is_readonly = fs.ends_with(":ro");
            let mut clean_fs = fs.to_string();
            if is_readonly {
                clean_fs = clean_fs.replace(":ro", "");
            }

            let label = if clean_fs == "home" {
                "Home directory".to_string()
            } else if clean_fs == "host" {
                "Entire filesystem (very broad)".to_string()
            } else if clean_fs == "xdg-download" {
                "Downloads folder".to_string()
            } else if clean_fs == "xdg-documents" {
                "Documents folder".to_string()
            } else if clean_fs == "xdg-music" {
                "Music folder".to_string()
            } else if clean_fs == "xdg-pictures" {
                "Pictures folder".to_string()
            } else if clean_fs == "xdg-videos" {
                "Videos folder".to_string()
            } else {
                clean_fs.clone()
            };

            let full_desc = if is_readonly {
                format!("{} (read-only)", label)
            } else {
                format!("{} (read-write)", label)
            };

            permissions.push(Permission {
                category: PermissionCategory::Filesystem,
                description: full_desc,
                source_mechanism: "flatpak".to_string(),
                raw: filesystems.clone(),
            });
        }
    }

    let mut profile = AppProfile::new(app_id.to_string());
    profile.permissions = permissions;

    Ok(profile)
}
