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

fn trans_raw_output(input: &str) -> Vec<Perm> {
    let mut results = Vec::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.contains("network") {
            results.push(Perm {
                cat: PermCat::Network,
                desc: "Network access".to_string(),
                source_mech: "pacman".to_string(),
                raw: line.to_string(),
            })
        }
    }
    results
}

fn fetch_app_data(app_id: &str) -> Result<String, CollectorError> {
    let output = Command::new("pacman")
        .arg("-Qi")
        .arg(app_id)
        .output()
        .map_err(|_| CollectorError::NotInst(format!("{} is not installed", app_id)))?;
    if !output.status.success() {
        return Err(CollectorError::CmdErr(format!(
            "pacman -Qi failed for {}",
            app_id
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
                let permissions = trans_raw_output(&raw_data);
                let mut profile = AppProf::new(id);
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
