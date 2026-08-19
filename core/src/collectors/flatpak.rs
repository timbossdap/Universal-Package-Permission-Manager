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
        return Err(CollectorError::CmdErr(
            "flatpak command failed".into(),
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let ids = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
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
                source_mech: "flatpak".to_string(),
                raw: line.to_string(),
            })
        }
    }
    results
}
fn fetch_app_data(app_id: &str) -> Result<String, CollectorError> {
    let output = Command::new("flatpak")
        .arg("info")
        .arg("--show=permissions")
        .arg(app_id)
        .output()
        .map_err(|_| CollectorError::NotInst(format!("{} is not installed", app_id)))?;
    if !output.status.success() {
        return Err(CollectorError::CmdErr(format!(
            "flatpak info failed for {}",
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
