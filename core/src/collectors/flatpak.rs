use std::collections::HashMap;
use std::process::Command;

use crate::AppProfile;
use crate::Permission;
use crate::PermissionCategory;
// lists all the installed flatpak apps
fn list_app_ids() -> Result<Vec<String>, CollectorError> {
    let output = Command::new("flatpak")
        .arg("list")
        .arg("--app")
        .arg("--columns=application")
        .output()
        .map_err(|_| CollectorError::NotInstalled("flatpak".to_string()))?;
    if !output.status.success() {
        return Err(CollectorError::CommandFailed("flatpak command failed".into()));
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let ids = text.lines().filter(|l| !l.trim().is_empty()).map(|l| l.trim().to_string()).collect();
    Ok(ids)
}
// translates the raw output of flatpak info into a vector of Permission structs
fn trans_raw_output(input: &str) -> Vec<Permission> {
    let mut results = Vec::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.contains("network") {
            results.push(Permission {
                category: PermissionCategory::Network,
                description: "Network access".to_string(),
                source_mechanism: "flatpak".to_string(),
                raw: line.to_string(),
            })
        }
    }
    results
}
// fetches the raw data for a given app id using flatpak info
fn fetch_app_data(app_id: &str) -> Result<String, CollectorError> {
    let output = Command::new("flatpak")
        .arg("info")
        .arg("--show=permissions")
        .arg(app_id)
        .output()
        .map_err(|_| CollectorError::NotInstalled(format!("{} is not installed", app_id)))?;
    if !output.status.success() {
        return Err(CollectorError::CommandFailed(format!("flatpak info failed for {}", app_id)));
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(text)
}
// collects app profiles by fetching data for each app id and translating the raw output
pub fn collect(&self) -> Result<Vec<AppProfile>, String> {
    let app_ids = list_app_ids().map_err(|e| e.to_string())?;
    for id in app_ids{
        let raw_data = fetch_raw_data
    }

}
