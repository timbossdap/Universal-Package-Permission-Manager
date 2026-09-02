use std::collections::HashMap;
use std::process::Command;

use crate::AppProf;
use crate::CollectorError;
use crate::Perm;
use crate::PermCat;

fn list_app_ids() -> Result<Vec<String>, CollectorError> {
    let output = Command::new("pacman")
        .arg("-Q")
        .output()
        .map_err(|_| CollectorError::NotInst("pacman".to_string()))?;
    if !output.status.success() {
        return Err(CollectorError::CmdErr("pacman command failed".into()));
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let ids = text
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.split_whitespace().next().unwrap_or("").trim().to_string())
        .collect();
    Ok(ids)
}

fn describe_path(path: &str) -> Option<(PermCat, String)> {
    let p = path;
    // .socket units specifically mean "opens a listening socket" - check
    // this before the generic systemd-directory branch below, since it's
    // a more specific (and more Network-flavored) claim than a plain service
    if p.ends_with(".socket")
        && (p.starts_with("/usr/lib/systemd/system/") || p.starts_with("/etc/systemd/system/"))
    {
        Some((
            PermCat::Network,
            "Installs a systemd socket unit (opens a listening socket for network/IPC activation)"
                .to_string(),
        ))
    } else if p.starts_with("/usr/lib/systemd/system/") || p.starts_with("/etc/systemd/system/") {
        Some((
            PermCat::System,
            "Installs a systemd service (can run privileged/background processes)".to_string(),
        ))
    } else if p.starts_with("/usr/lib/systemd/user/") {
        Some((
            PermCat::System,
            "Installs a user-level systemd service".to_string(),
        ))
    } else if p.starts_with("/usr/lib/udev/rules.d/") || p.starts_with("/etc/udev/rules.d/") {
        Some((
            PermCat::Hardware,
            "Installs udev rules (hardware/device access rules)".to_string(),
        ))
    } else if p.contains("/polkit-1/actions/") || p.contains("/polkit-1/rules.d/") {
        Some((
            PermCat::System,
            "Installs a polkit policy (can grant privilege escalation)".to_string(),
        ))
    } else if p.starts_with("/etc/sudoers.d/") {
        Some((
            PermCat::System,
            "Installs a sudoers rule (elevated command access)".to_string(),
        ))
    } else if p.starts_with("/usr/share/dbus-1/system-services/")
        || p.starts_with("/etc/dbus-1/system.d/")
    {
        Some((
            PermCat::System,
            "Installs a D-Bus system service (privileged IPC)".to_string(),
        ))
    } else if p.starts_with("/etc/pam.d/") {
        Some((
            PermCat::System,
            "Installs a PAM module (affects authentication)".to_string(),
        ))
    } else if p.starts_with("/usr/lib/modules-load.d/") || p.starts_with("/etc/modules-load.d/") {
        Some((PermCat::System, "Loads a kernel module at boot".to_string()))
    } else if p.starts_with("/usr/lib/sysctl.d/") || p.starts_with("/etc/sysctl.d/") {
        Some((
            PermCat::System,
            "Modifies kernel parameters (sysctl)".to_string(),
        ))
    } else if p.starts_with("/etc/NetworkManager/") || p.contains("/NetworkManager/dispatcher.d/") {
        Some((PermCat::Network, "Hooks into NetworkManager".to_string()))
    } else if p.starts_with("/etc/cron.") || p.contains("/cron.d/") {
        Some((
            PermCat::System,
            "Installs a cron job (scheduled execution)".to_string(),
        ))
    } else if p.starts_with("/etc/apparmor.d/") {
        Some((PermCat::System, "Installs an AppArmor profile".to_string()))
    } else if p.starts_with("/usr/share/selinux/") {
        Some((
            PermCat::System,
            "Installs an SELinux policy module".to_string(),
        ))
    } else if p.starts_with("/etc/profile.d/") {
        Some((
            PermCat::System,
            "Modifies the shell environment for all users at login".to_string(),
        ))
    } else if p.starts_with("/etc/xdg/autostart/") {
        Some((
            PermCat::Desktop,
            "Registers an autostart entry (runs automatically at login)".to_string(),
        ))
    } else if p.starts_with("/usr/share/dbus-1/services/") {
        Some((
            PermCat::Desktop,
            "Installs a D-Bus session service".to_string(),
        ))
    } else if p.starts_with("/usr/lib/tmpfiles.d/") || p.starts_with("/etc/tmpfiles.d/") {
        Some((
            PermCat::System,
            "Creates/modifies files or directories at boot (tmpfiles.d)".to_string(),
        ))
    } else {
        None
    }
}

fn check_capabilities(path: &str) -> Option<(PermCat, String)> {
    let output = Command::new("getcap").arg(path).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let caps = text.split_once(' ').map(|(_, c)| c.trim()).unwrap_or(text);
    if caps.is_empty() {
        return None;
    }
    Some((
        PermCat::System,
        format!("Has Linux capabilities: {caps} (elevated privilege without setuid)"),
    ))
}

fn check_binary_privileges(path: &str) -> Vec<(PermCat, String)> {
    use std::os::unix::fs::PermissionsExt;
    let mut results = Vec::new();

    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return results;
    };
    if meta.file_type().is_symlink() {
        return results;
    }
    let mode = meta.permissions().mode();
    let setuid = mode & 0o4000 != 0;
    let setgid = mode & 0o2000 != 0;
    match (setuid, setgid) {
        (true, true) => results.push((
            PermCat::System,
            "setuid+setgid binary (runs as file owner/group)".to_string(),
        )),
        (true, false) => results.push((
            PermCat::System,
            "setuid binary (runs as file owner, often root)".to_string(),
        )),
        (false, true) => results.push((
            PermCat::System,
            "setgid binary (runs as file group)".to_string(),
        )),
        (false, false) => {}
    }

    let executable = mode & 0o111 != 0;
    if executable {
        if let Some(cap) = check_capabilities(path) {
            results.push(cap);
        }
    }

    results
}

fn list_all_owned_files() -> Result<HashMap<String, Vec<String>>, CollectorError> {
    let output = Command::new("pacman")
        .arg("-Ql")
        .output()
        .map_err(|_| CollectorError::CmdErr("failed to run pacman -Ql".to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(CollectorError::CmdErr(format!(
            "pacman -Ql failed: {stderr}"
        )));
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for line in text.lines() {
        // Format: "pkgname /some/path"
        let Some((pkg, path)) = line.split_once(' ') else {
            continue;
        };
        let path = path.trim();
        if path.is_empty() || path.ends_with('/') {
            continue;
        }
        map.entry(pkg.to_string())
            .or_default()
            .push(path.to_string());
    }
    Ok(map)
}

fn parse_permissions(files: &[String]) -> Vec<Perm> {
    let mut results = Vec::new();
    for path in files {
        if let Some((cat, desc)) = describe_path(path) {
            results.push(Perm {
                cat,
                desc,
                source_mech: "pacman".to_string(),
                raw: path.clone(),
            });
        }
        for (cat, desc) in check_binary_privileges(path) {
            results.push(Perm {
                cat,
                desc,
                source_mech: "pacman".to_string(),
                raw: path.clone(),
            });
        }
    }
    results
}

const PRIVILEGED_DEPS: &[(&str, &str)] = &[
    (
        "polkit",
        "Depends on polkit (can request privilege escalation via authentication prompts)",
    ),
    (
        "systemd",
        "Depends on systemd (can register services, timers, or other units)",
    ),
    (
        "sudo",
        "Depends on sudo (can request elevated command execution)",
    ),
    ("dbus", "Depends on dbus (privileged IPC access)"),
    ("pam", "Depends on PAM (can affect authentication)"),
    (
        "firewalld",
        "Depends on firewalld (can alter firewall rules)",
    ),
    ("bluez", "Depends on bluez (Bluetooth stack access)"),
    ("avahi", "Depends on avahi (network service discovery)"),
    ("cups", "Depends on CUPS (printing system access)"),
    (
        "docker",
        "Depends on docker (container runtime - effectively root-equivalent access)",
    ),
];

// per-package info pulled from a single `pacman -Qi` pass - depends and
// install-script are both fields on the same block per package, so it's
// free to grab both here instead of running pacman -Qi twice
struct PkgInfo {
    depends: Vec<String>,
    has_install_script: bool,
}

fn get_all_pkg_info() -> HashMap<String, PkgInfo> {
    let output = match Command::new("pacman").arg("-Qi").output() {
        Ok(o) if o.status.success() => o,
        _ => return HashMap::new(),
    };
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let mut map: HashMap<String, PkgInfo> = HashMap::new();
    let mut current_name: Option<String> = None;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("Name") {
            let rest = rest
                .trim_start_matches(|c: char| c == ' ' || c == ':')
                .trim();
            if !rest.is_empty() {
                current_name = Some(rest.to_string());
                map.entry(rest.to_string()).or_insert(PkgInfo {
                    depends: Vec::new(),
                    has_install_script: false,
                });
            }
        } else if let Some(rest) = line.strip_prefix("Depends On") {
            let rest = rest
                .trim_start_matches(|c: char| c == ' ' || c == ':')
                .trim();
            let Some(name) = current_name.clone() else {
                continue;
            };
            let deps: Vec<String> = if rest.is_empty() || rest == "None" {
                Vec::new()
            } else {
                rest.split_whitespace()
                    .map(|d| d.split(['>', '<', '=']).next().unwrap_or(d).to_string())
                    .collect()
            };
            if let Some(entry) = map.get_mut(&name) {
                entry.depends = deps;
            }
        } else if let Some(rest) = line.strip_prefix("Install Script") {
            let rest = rest
                .trim_start_matches(|c: char| c == ' ' || c == ':')
                .trim();
            let Some(name) = current_name.clone() else {
                continue;
            };
            if let Some(entry) = map.get_mut(&name) {
                entry.has_install_script = rest.eq_ignore_ascii_case("Yes");
            }
        }
    }
    map
}

fn check_dependencies(app_id: &str, pkg_info: &HashMap<String, PkgInfo>) -> Vec<Perm> {
    let empty: Vec<String> = Vec::new();
    let deps = pkg_info.get(app_id).map(|info| &info.depends).unwrap_or(&empty);
    let mut results = Vec::new();
    for (needle, desc) in PRIVILEGED_DEPS {
        if deps.iter().any(|d| d == needle) {
            results.push(Perm {
                cat: PermCat::System,
                desc: desc.to_string(),
                source_mech: "pacman-deps".to_string(),
                raw: format!("Depends On: {needle}"),
            });
        }
    }
    results
}

// install scriptlets (.install) run arbitrary code as root during
// install/upgrade/remove - this is pacman's rough equivalent of a Flatpak
// permission, and the single biggest trust decision a package makes, so
// it's worth surfacing even without inspecting what the script actually does
fn check_install_script(app_id: &str, pkg_info: &HashMap<String, PkgInfo>) -> Option<Perm> {
    let info = pkg_info.get(app_id)?;
    if !info.has_install_script {
        return None;
    }
    Some(Perm {
        cat: PermCat::System,
        desc: "Runs a pacman install script (arbitrary code as root during install/upgrade/remove)"
            .to_string(),
        source_mech: "pacman-install-script".to_string(),
        raw: format!("{app_id}.install"),
    })
}

// maps package name -> its local metadata dir under /var/lib/pacman/local,
// so check_group_hints can find each package's .install script by name
fn build_local_dir_map() -> HashMap<String, std::path::PathBuf> {
    let mut map = HashMap::new();
    let Ok(entries) = std::fs::read_dir("/var/lib/pacman/local") else {
        return map;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        // dir names are "pkgname-pkgver-pkgrel" - split from the right so
        // pkgnames that themselves contain hyphens still work
        let parts: Vec<&str> = name.rsplitn(3, '-').collect();
        if parts.len() == 3 {
            map.insert(parts[2].to_string(), path);
        }
    }
    map
}

// hardware access on Arch is usually gated by unix group membership
// (video, dialout, etc.) rather than a sandbox permission, and install
// scripts often tell the user to `usermod -aG <group> $USER` in their
// post-install message. this is a heuristic (greps the script text) not
// ground truth - it's possible for a script to mention a group name
// without actually adding the user to it, so treat this as a hint
const HARDWARE_GROUPS: &[&str] = &[
    "video", "audio", "dialout", "wireshark", "docker", "scanner", "lp", "kvm", "render",
];

fn check_group_hints(pkg: &str, local_dirs: &HashMap<String, std::path::PathBuf>) -> Option<Perm> {
    let dir = local_dirs.get(pkg)?;
    let content = std::fs::read_to_string(dir.join("install")).ok()?;
    let group = HARDWARE_GROUPS
        .iter()
        .find(|g| content.contains("usermod") && content.contains(*g))?;
    Some(Perm {
        cat: PermCat::Hardware,
        desc: format!(
            "Install script suggests adding your user to the '{group}' group (grants hardware access outside pacman's own permission model)"
        ),
        source_mech: "pacman-install-script".to_string(),
        raw: dir.join("install").display().to_string(),
    })
}

pub fn collect() -> Result<Vec<AppProf>, String> {
    let app_ids = list_app_ids().map_err(|e| e.to_string())?;
    let owned_files = list_all_owned_files().map_err(|e| e.to_string())?;
    let pkg_info = get_all_pkg_info();
    let local_dirs = build_local_dir_map();

    let mut profiles = Vec::new();
    for id in app_ids {
        match owned_files.get(&id) {
            Some(files) => {
                let mut permissions = parse_permissions(files);
                permissions.extend(check_dependencies(&id, &pkg_info));
                permissions.extend(check_install_script(&id, &pkg_info));
                permissions.extend(check_group_hints(&id, &local_dirs));
                let mut profile = AppProf::new(id);
                profile.permissions = permissions;
                profiles.push(profile);
            }
            None => {
                println!(
                    "Error: Could not get details for {id}, reason: no files found in pacman -Ql"
                );
            }
        }
    }
    Ok(profiles)
}
