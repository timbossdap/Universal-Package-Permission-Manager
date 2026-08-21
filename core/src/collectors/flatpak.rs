use std::process::Command;

use crate::AppProf;
use crate::CollectorError;
use crate::Perm;
use crate::PermCat;

fn list_app_ids() -> Result<Vec<String>, CollectorError> {
    let output = Command::new("flatpak")
        .arg("list")
        .arg("--app")
        .arg("--columns=application")
        .output()
        .map_err(|_| CollectorError::NotInst("flatpak".to_string()))?;
    if !output.status.success() {
        return Err(CollectorError::CmdErr("flatpak command failed".into()));
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let ids = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter(|l| !l.starts_with("Name:") && !l.starts_with("Application ID"))
        .map(|l| l.trim().to_string())
        .collect();
    Ok(ids)
}

/// Maps one raw [Context] token (e.g. "network" under "shared=") to a
/// (category, human description) pair. This is the mapping table that
/// does the actual "Flatseal-style" translation work.
fn describe(key: &str, value: &str) -> Option<(PermCat, String)> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    match key {
        "shared" => match value {
            "network" => Some((PermCat::Network, "Network access".to_string())),
            "ipc" => Some((PermCat::System, "Inter-process communication".to_string())),
            other => Some((PermCat::System, format!("Shared: {other}"))),
        },
        "sockets" => match value {
            "x11" => Some((PermCat::Desktop, "X11 windowing system access".to_string())),
            "wayland" => Some((PermCat::Desktop, "Wayland windowing system access".to_string())),
            "fallback-x11" => Some((PermCat::Desktop, "Fallback X11 access".to_string())),
            "pulseaudio" => Some((PermCat::Hardware, "Audio playback/recording (PulseAudio)".to_string())),
            "session-bus" => Some((PermCat::System, "Full session D-Bus access".to_string())),
            "system-bus" => Some((PermCat::System, "Full system D-Bus access".to_string())),
            "ssh-auth" => Some((PermCat::System, "SSH agent access".to_string())),
            "pcsc" => Some((PermCat::Hardware, "Smart card access".to_string())),
            "cups" => Some((PermCat::System, "Printing (CUPS) access".to_string())),
            other => Some((PermCat::System, format!("Socket: {other}"))),
        },
        "devices" => match value {
            "all" => Some((PermCat::Hardware, "All devices (unrestricted hardware access)".to_string())),
            "dri" => Some((PermCat::Hardware, "GPU acceleration (DRI)".to_string())),
            "kvm" => Some((PermCat::Hardware, "Virtualization (KVM)".to_string())),
            "shm" => Some((PermCat::Hardware, "Shared memory device access".to_string())),
            "input" => Some((PermCat::Hardware, "Input devices (keyboard/mouse/etc)".to_string())),
            "usb" => Some((PermCat::Hardware, "USB device access".to_string())),
            other => Some((PermCat::Hardware, format!("Device: {other}"))),
        },
        "filesystems" => match value {
            "host" => Some((PermCat::Filesystem, "Full filesystem access".to_string())),
            "home" | "~" => Some((PermCat::Filesystem, "Home directory access".to_string())),
            "host-os" => Some((PermCat::Filesystem, "Host OS files access".to_string())),
            "host-etc" => Some((PermCat::Filesystem, "System configuration (/etc) access".to_string())),
            "xdg-download" => Some((PermCat::Filesystem, "Downloads folder access".to_string())),
            "xdg-documents" => Some((PermCat::Filesystem, "Documents folder access".to_string())),
            "xdg-pictures" => Some((PermCat::Filesystem, "Pictures folder access".to_string())),
            "xdg-music" => Some((PermCat::Filesystem, "Music folder access".to_string())),
            "xdg-videos" => Some((PermCat::Filesystem, "Videos folder access".to_string())),
            "xdg-run" => Some((PermCat::Filesystem, "Runtime directory access".to_string())),
            "xdg-config" => Some((PermCat::Filesystem, "Config directory access".to_string())),
            other => Some((PermCat::Filesystem, format!("Filesystem: {other}"))),
        },
        "features" => Some((PermCat::System, format!("Feature: {value}"))),
        _ => None,
    }
}

/// Parses `flatpak info --show-permissions` output. The relevant part
/// looks like:
///
/// [Context]
/// shared=network;ipc;
/// sockets=x11;wayland;pulseaudio;
/// devices=dri;
/// filesystems=host;xdg-download;
///
/// [Session Bus Policy]
/// org.freedesktop.Notifications=talk
///
/// We only read the [Context] block for now -- that covers the five main
/// permission categories. The bus policy sections are a natural next step.
fn parse_permissions(input: &str) -> Vec<Perm> {
    let mut results = Vec::new();
    let mut in_context = false;

    for raw_line in input.lines() {
        let line = raw_line.trim();

        if line.is_empty() {
            continue;
        }

        // Section headers look like "[Context]" or "[Session Bus Policy]".
        if line.starts_with('[') && line.ends_with(']') {
            in_context = line == "[Context]";
            continue;
        }

        if !in_context {
            continue;
        }

        // `split_once` gives us everything before/after the first '='.
        // `let ... else { continue }` skips lines that don't have an '='
        // (e.g. stray blank lines) instead of panicking.
        let Some((key, values)) = line.split_once('=') else {
            continue;
        };

        // Values are semicolon-separated, e.g. "x11;wayland;pulseaudio;"
        for value in values.split(';') {
            if let Some((cat, desc)) = describe(key.trim(), value) {
                results.push(Perm {
                    cat,
                    desc,
                    source_mech: "flatpak".to_string(),
                    raw: line.to_string(),
                });
            }
        }
    }

    results
}

fn fetch_app_data(app_id: &str) -> Result<String, CollectorError> {
    let output = Command::new("flatpak")
        .arg("info")
        .arg("--show-permissions")
        .arg(app_id)
        .output()
        .map_err(|_| CollectorError::NotInst(format!("{app_id} is not installed")))?;

    if !output.status.success() {
        // Surface flatpak's own error text instead of swallowing it -- this
        // is what would've said "Unknown option" and pointed at the bug.
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(CollectorError::CmdErr(format!(
            "flatpak info --show-permissions failed for {app_id}: {stderr}"
        )));
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(text)
}

pub fn collect() -> Result<Vec<AppProf>, String> {
    let app_ids = list_app_ids().map_err(|e| e.to_string())?;
    let mut profiles = Vec::new();
    for id in app_ids {
        match fetch_app_data(&id) {
            Ok(raw_data) => {
                let permissions = parse_permissions(&raw_data);
                let mut profile = AppProf::new(id.clone()); // Need to clone the ID string
                profile.permissions = permissions;
                profiles.push(profile);
            }
            Err(e) => {
                println!("Error: Could not get details for {}, reason: {}", id, e);
            }
        }
    }
    Ok(profiles)
}
