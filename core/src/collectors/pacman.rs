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
        .map(|l| {
            // pacman -Q output format: "package-name version"
            l.split_whitespace()
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        })
        .collect();
    Ok(ids)
}

/// Pacman packages aren't sandboxed the way Flatpak apps are, so there's no
/// `--show-permissions` equivalent to parse. The closest meaningful analog
/// is auditing *where a package installs files* -- a package that drops a
/// systemd unit, a polkit rule, or a sudoers entry is granting itself (or a
/// service it runs) privilege the user should know about, even though
/// pacman itself has no concept of "permissions".
fn describe_path(path: &str) -> Option<(PermCat, String)> {
    let p = path;
    if p.starts_with("/usr/lib/systemd/system/") || p.starts_with("/etc/systemd/system/") {
        Some((PermCat::System, "Installs a systemd service (can run privileged/background processes)".to_string()))
    } else if p.starts_with("/usr/lib/systemd/user/") {
        Some((PermCat::System, "Installs a user-level systemd service".to_string()))
    } else if p.starts_with("/usr/lib/udev/rules.d/") || p.starts_with("/etc/udev/rules.d/") {
        Some((PermCat::Hardware, "Installs udev rules (hardware/device access rules)".to_string()))
    } else if p.contains("/polkit-1/actions/") || p.contains("/polkit-1/rules.d/") {
        Some((PermCat::System, "Installs a polkit policy (can grant privilege escalation)".to_string()))
    } else if p.starts_with("/etc/sudoers.d/") {
        Some((PermCat::System, "Installs a sudoers rule (elevated command access)".to_string()))
    } else if p.starts_with("/usr/share/dbus-1/system-services/") || p.starts_with("/etc/dbus-1/system.d/") {
        Some((PermCat::System, "Installs a D-Bus system service (privileged IPC)".to_string()))
    } else if p.starts_with("/etc/pam.d/") {
        Some((PermCat::System, "Installs a PAM module (affects authentication)".to_string()))
    } else if p.starts_with("/usr/lib/modules-load.d/") || p.starts_with("/etc/modules-load.d/") {
        Some((PermCat::System, "Loads a kernel module at boot".to_string()))
    } else if p.starts_with("/usr/lib/sysctl.d/") || p.starts_with("/etc/sysctl.d/") {
        Some((PermCat::System, "Modifies kernel parameters (sysctl)".to_string()))
    } else if p.starts_with("/etc/NetworkManager/") || p.contains("/NetworkManager/dispatcher.d/") {
        Some((PermCat::Network, "Hooks into NetworkManager".to_string()))
    } else if p.starts_with("/etc/cron.") || p.contains("/cron.d/") {
        Some((PermCat::System, "Installs a cron job (scheduled execution)".to_string()))
    } else {
        None
    }
}

/// Checks the actual file mode bits for setuid/setgid, since those grant
/// real privilege escalation regardless of install path. This is a plain
/// stat, not a subprocess spawn, so it's cheap to run per file.
fn check_setuid_setgid(path: &str) -> Option<(PermCat, String)> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::symlink_metadata(path).ok()?;
    if meta.file_type().is_symlink() {
        return None;
    }
    let mode = meta.permissions().mode();
    let setuid = mode & 0o4000 != 0;
    let setgid = mode & 0o2000 != 0;
    match (setuid, setgid) {
        (true, true) => Some((PermCat::System, "setuid+setgid binary (runs as file owner/group)".to_string())),
        (true, false) => Some((PermCat::System, "setuid binary (runs as file owner, often root)".to_string())),
        (false, true) => Some((PermCat::System, "setgid binary (runs as file group)".to_string())),
        (false, false) => None,
    }
}

/// Lists every file pacman recorded as owned by `app_id` via `pacman -Ql`.
/// Output format is "pkgname /abs/path" per line; directories end in '/'
/// and are filtered out since only files carry meaningful permission bits.
fn list_owned_files(app_id: &str) -> Result<Vec<String>, CollectorError> {
    let output = Command::new("pacman")
        .arg("-Ql")
        .arg(app_id)
        .output()
        .map_err(|_| CollectorError::NotInst(format!("{app_id} is not installed")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(CollectorError::CmdErr(format!(
            "pacman -Ql failed for {app_id}: {stderr}"
        )));
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let files = text
        .lines()
        .filter_map(|line| line.split_once(' '))
        .map(|(_, path)| path.trim().to_string())
        .filter(|path| !path.ends_with('/'))
        .collect();
    Ok(files)
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
        if let Some((cat, desc)) = check_setuid_setgid(path) {
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

pub fn collect() -> Result<Vec<AppProf>, String> {
    let app_ids = list_app_ids().map_err(|e| e.to_string())?;
    let mut profiles = Vec::new();
    for id in app_ids {
        match list_owned_files(&id) {
            Ok(files) => {
                let permissions = parse_permissions(&files);
                let mut profile = AppProf::new(id);
                profile.permissions = permissions;
                profiles.push(profile);
            }
            Err(e) => {
                println!("Error: Could not get details for {id}, reason: {e}");
            }
        }
    }
    Ok(profiles)
}
