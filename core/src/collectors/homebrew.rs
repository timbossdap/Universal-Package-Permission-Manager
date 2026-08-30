use std::collections::HashMap;
use std::process::Command;

use crate::AppProf;
use crate::CollectorError;
use crate::Perm;
use crate::PermCat;

// --- Minimal Zero-Dependency JSON Parser ---

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(o) => o.get(key),
            _ => None,
        }
    }
}

struct JsonParser<'a> {
    chars: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        JsonParser {
            chars: input.as_bytes(),
            pos: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() {
            match self.chars[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        if self.pos < self.chars.len() {
            Some(self.chars[self.pos])
        } else {
            None
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let b = self.peek().ok_or_else(|| "Unexpected end of input".to_string())?;
        match b {
            b'n' => self.parse_null(),
            b't' | b'f' => self.parse_bool(),
            b'"' => self.parse_string().map(JsonValue::String),
            b'[' => self.parse_array(),
            b'{' => self.parse_object(),
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => Err(format!("Unexpected character: {} at pos {}", b as char, self.pos)),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.chars[self.pos..].starts_with(b"null") {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err(format!("Expected 'null' at pos {}", self.pos))
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.chars[self.pos..].starts_with(b"true") {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.chars[self.pos..].starts_with(b"false") {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err(format!("Expected boolean at pos {}", self.pos))
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        if self.peek() != Some(b'"') {
            return Err(format!("Expected '\"' at pos {}", self.pos));
        }
        self.pos += 1; // consume opening quote
        let mut result = String::new();
        while self.pos < self.chars.len() {
            let b = self.chars[self.pos];
            self.pos += 1;
            match b {
                b'"' => return Ok(result),
                b'\\' => {
                    if self.pos >= self.chars.len() {
                        return Err("Unterminated escape sequence".to_string());
                    }
                    let esc = self.chars[self.pos];
                    self.pos += 1;
                    match esc {
                        b'"' => result.push('"'),
                        b'\\' => result.push('\\'),
                        b'/' => result.push('/'),
                        b'b' => result.push('\x08'),
                        b'f' => result.push('\x0C'),
                        b'n' => result.push('\n'),
                        b'r' => result.push('\r'),
                        b't' => result.push('\t'),
                        b'u' => {
                            if self.pos + 4 > self.chars.len() {
                                return Err("Truncated unicode escape".to_string());
                            }
                            let hex_str = std::str::from_utf8(&self.chars[self.pos..self.pos + 4])
                                .map_err(|e| e.to_string())?;
                            self.pos += 4;
                            let code = u32::from_str_radix(hex_str, 16)
                                .map_err(|e| format!("Invalid hex: {e}"))?;
                            if let Some(c) = char::from_u32(code) {
                                result.push(c);
                            } else {
                                result.push('\u{FFFD}');
                            }
                        }
                        other => {
                            result.push(other as char);
                        }
                    }
                }
                _ => {
                    result.push(b as char);
                }
            }
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos < self.chars.len() && self.chars[self.pos] == b'.' {
            self.pos += 1;
            while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        if self.pos < self.chars.len()
            && (self.chars[self.pos] == b'e' || self.chars[self.pos] == b'E')
        {
            self.pos += 1;
            if self.pos < self.chars.len()
                && (self.chars[self.pos] == b'+' || self.chars[self.pos] == b'-')
            {
                self.pos += 1;
            }
            while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        let num_str = std::str::from_utf8(&self.chars[start..self.pos])
            .map_err(|e| e.to_string())?;
        let num: f64 = num_str.parse().map_err(|e: std::num::ParseFloatError| e.to_string())?;
        Ok(JsonValue::Number(num))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        if self.peek() != Some(b'[') {
            return Err(format!("Expected '[' at pos {}", self.pos));
        }
        self.pos += 1;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            let val = self.parse_value()?;
            items.push(val);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                other => {
                    return Err(format!(
                        "Expected ',' or ']' in array, got {:?}",
                        other.map(|c| c as char)
                    ));
                }
            }
        }
        Ok(JsonValue::Array(items))
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        if self.peek() != Some(b'{') {
            return Err(format!("Expected '{{' at pos {}", self.pos));
        }
        self.pos += 1;
        let mut map = HashMap::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(map));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.peek() != Some(b':') {
                return Err(format!("Expected ':' after key at pos {}", self.pos));
            }
            self.pos += 1;
            let val = self.parse_value()?;
            map.insert(key, val);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    self.skip_whitespace();
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                other => {
                    return Err(format!(
                        "Expected ',' or '}}' in object, got {:?}",
                        other.map(|c| c as char)
                    ));
                }
            }
        }
        Ok(JsonValue::Object(map))
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    let mut parser = JsonParser::new(input);
    let val = parser.parse_value()?;
    parser.skip_whitespace();
    Ok(val)
}

// --- Homebrew Permission Analysis ---

const PRIVILEGED_DEPS: &[(&str, &str)] = &[
    (
        "sudo",
        "Depends on sudo (can request elevated command execution)",
    ),
    (
        "dbus",
        "Depends on dbus (inter-process communication / privileged system bus)",
    ),
    (
        "polkit",
        "Depends on polkit (can grant privilege escalation)",
    ),
    (
        "pam",
        "Depends on PAM (affects system authentication)",
    ),
    (
        "openpam",
        "Depends on OpenPAM (affects system authentication)",
    ),
    (
        "tccutil",
        "Depends on tccutil (modifies macOS Transparency, Consent, and Control permissions)",
    ),
    (
        "shadow",
        "Depends on shadow (manages user accounts and passwords)",
    ),
    (
        "wireguard-tools",
        "Depends on WireGuard tools (configures network tunnels / interfaces)",
    ),
    (
        "tailscale",
        "Depends on Tailscale (configures network tunnels / VPN)",
    ),
];

fn analyze_caveats(caveats_text: &str, source_mech: &str) -> Vec<Perm> {
    let mut perms = Vec::new();
    let lower = caveats_text.to_lowercase();

    if lower.contains("launchdaemons") || (lower.contains("sudo") && lower.contains("launchctl")) {
        perms.push(Perm {
            cat: PermCat::System,
            desc: "Requires root LaunchDaemon setup (sudo launchctl / /Library/LaunchDaemons)".to_string(),
            source_mech: source_mech.to_string(),
            raw: "caveats: LaunchDaemon".to_string(),
        });
    } else if lower.contains("sudo") || lower.contains("root") {
        perms.push(Perm {
            cat: PermCat::System,
            desc: "Requires elevated root privileges (sudo instructions in caveats)".to_string(),
            source_mech: source_mech.to_string(),
            raw: "caveats: sudo".to_string(),
        });
    }

    if lower.contains("accessibility") {
        perms.push(Perm {
            cat: PermCat::Desktop,
            desc: "Requests macOS Accessibility permissions (can control UI / monitor events)".to_string(),
            source_mech: source_mech.to_string(),
            raw: "caveats: Accessibility".to_string(),
        });
    }

    if lower.contains("full disk access") {
        perms.push(Perm {
            cat: PermCat::Filesystem,
            desc: "Requests macOS Full Disk Access permissions".to_string(),
            source_mech: source_mech.to_string(),
            raw: "caveats: Full Disk Access".to_string(),
        });
    }

    if lower.contains("screen recording") || lower.contains("screencapture") {
        perms.push(Perm {
            cat: PermCat::Desktop,
            desc: "Requests macOS Screen Recording permissions (can capture display)".to_string(),
            source_mech: source_mech.to_string(),
            raw: "caveats: Screen Recording".to_string(),
        });
    }

    if lower.contains("input monitoring") {
        perms.push(Perm {
            cat: PermCat::Hardware,
            desc: "Requests macOS Input Monitoring permissions (can monitor keystrokes/devices)".to_string(),
            source_mech: source_mech.to_string(),
            raw: "caveats: Input Monitoring".to_string(),
        });
    }

    if lower.contains("kernel extension") || lower.contains("kext") {
        perms.push(Perm {
            cat: PermCat::Hardware,
            desc: "Requests macOS Kernel Extension (kext) approval".to_string(),
            source_mech: source_mech.to_string(),
            raw: "caveats: Kernel Extension".to_string(),
        });
    }

    perms
}

fn describe_zap_path(path: &str) -> Option<(PermCat, String)> {
    let p = path.trim();
    if p.starts_with("~/Library/LaunchAgents") || p.starts_with("/Library/LaunchAgents") {
        Some((
            PermCat::System,
            format!("Registers background LaunchAgent service ({p})"),
        ))
    } else if p.starts_with("/Library/LaunchDaemons") {
        Some((
            PermCat::System,
            format!("Registers root LaunchDaemon service ({p})"),
        ))
    } else if p.starts_with("~/Library/Application Support") {
        Some((
            PermCat::Filesystem,
            format!("Accesses user Application Support directory ({p})"),
        ))
    } else if p.starts_with("~/Library/Preferences") || p.starts_with("/Library/Preferences") {
        Some((
            PermCat::Filesystem,
            format!("Accesses user Preferences directory ({p})"),
        ))
    } else if p.starts_with("~/Library/Saved Application State") {
        Some((
            PermCat::Filesystem,
            format!("Accesses Saved Application State ({p})"),
        ))
    } else if p.starts_with("~/.config") || p.starts_with("~/.gemini") || p.starts_with("~/.") {
        Some((
            PermCat::Filesystem,
            format!("Accesses user dotfiles/configuration in home directory ({p})"),
        ))
    } else if p.starts_with("/Library/") || p.starts_with("/etc/") || p.starts_with("/var/") {
        Some((
            PermCat::Filesystem,
            format!("Accesses system files ({p})"),
        ))
    } else if !p.is_empty() {
        Some((
            PermCat::Filesystem,
            format!("Touches filesystem path ({p})"),
        ))
    } else {
        None
    }
}

fn parse_cask(cask: &JsonValue) -> Option<AppProf> {
    let token = cask.get("token")?.as_str()?.to_string();
    let mut permissions = Vec::new();
    let source_mech = "homebrew-cask".to_string();

    // Artifacts analysis
    if let Some(artifacts) = cask.get("artifacts").and_then(|v| v.as_array()) {
        for artifact in artifacts {
            if let Some(obj) = artifact.as_object() {
                for (key, val) in obj {
                    match key.as_str() {
                        "app" => {
                            let path_desc = val.as_array()
                                .and_then(|a| a.first())
                                .and_then(|v| v.as_str())
                                .unwrap_or("App bundle");
                            permissions.push(Perm {
                                cat: PermCat::Desktop,
                                desc: format!("Installs macOS GUI Application bundle ({path_desc})"),
                                source_mech: source_mech.clone(),
                                raw: format!("app: {path_desc}"),
                            });
                        }
                        "pkg" => {
                            let path_desc = val.as_array()
                                .and_then(|a| a.first())
                                .and_then(|v| v.as_str())
                                .unwrap_or(".pkg installer");
                            permissions.push(Perm {
                                cat: PermCat::System,
                                desc: format!("Installs via macOS Installer package (.pkg, can execute root scripts: {path_desc})"),
                                source_mech: source_mech.clone(),
                                raw: format!("pkg: {path_desc}"),
                            });
                        }
                        "installer" => {
                            permissions.push(Perm {
                                cat: PermCat::System,
                                desc: "Executes custom installer script / executable".to_string(),
                                source_mech: source_mech.clone(),
                                raw: "installer: script/manual".to_string(),
                            });
                        }
                        "kext" => {
                            let path_desc = val.as_array()
                                .and_then(|a| a.first())
                                .and_then(|v| v.as_str())
                                .unwrap_or("kext");
                            permissions.push(Perm {
                                cat: PermCat::Hardware,
                                desc: format!("Installs macOS Kernel Extension ({path_desc} - direct kernel-level execution)"),
                                source_mech: source_mech.clone(),
                                raw: format!("kext: {path_desc}"),
                            });
                        }
                        "launch_daemon" => {
                            permissions.push(Perm {
                                cat: PermCat::System,
                                desc: "Installs system LaunchDaemon (runs background service as root)".to_string(),
                                source_mech: source_mech.clone(),
                                raw: "launch_daemon".to_string(),
                            });
                        }
                        "service" => {
                            permissions.push(Perm {
                                cat: PermCat::System,
                                desc: "Installs background service".to_string(),
                                source_mech: source_mech.clone(),
                                raw: "service".to_string(),
                            });
                        }
                        "audio_unit_plugin" | "vst_plugin" | "vst3_plugin" => {
                            permissions.push(Perm {
                                cat: PermCat::Hardware,
                                desc: format!("Installs audio plugin ({key} - hardware audio processing)"),
                                source_mech: source_mech.clone(),
                                raw: key.clone(),
                            });
                        }
                        "input_method" => {
                            permissions.push(Perm {
                                cat: PermCat::Hardware,
                                desc: "Installs system input method (can monitor keyboard input)".to_string(),
                                source_mech: source_mech.clone(),
                                raw: "input_method".to_string(),
                            });
                        }
                        "screen_saver" => {
                            permissions.push(Perm {
                                cat: PermCat::Desktop,
                                desc: "Installs macOS screen saver".to_string(),
                                source_mech: source_mech.clone(),
                                raw: "screen_saver".to_string(),
                            });
                        }
                        "qlplugin" => {
                            permissions.push(Perm {
                                cat: PermCat::Desktop,
                                desc: "Installs QuickLook preview plugin".to_string(),
                                source_mech: source_mech.clone(),
                                raw: "qlplugin".to_string(),
                            });
                        }
                        "colorpicker" => {
                            permissions.push(Perm {
                                cat: PermCat::Desktop,
                                desc: "Installs system color picker plugin".to_string(),
                                source_mech: source_mech.clone(),
                                raw: "colorpicker".to_string(),
                            });
                        }
                        "font" => {
                            permissions.push(Perm {
                                cat: PermCat::Desktop,
                                desc: "Installs system font(s)".to_string(),
                                source_mech: source_mech.clone(),
                                raw: "font".to_string(),
                            });
                        }
                        "dictionary" => {
                            permissions.push(Perm {
                                cat: PermCat::Desktop,
                                desc: "Installs system dictionary".to_string(),
                                source_mech: source_mech.clone(),
                                raw: "dictionary".to_string(),
                            });
                        }
                        "binary" => {
                            let bin_name = val.as_array()
                                .and_then(|a| a.first())
                                .and_then(|v| v.as_str())
                                .unwrap_or("binary");
                            permissions.push(Perm {
                                cat: PermCat::System,
                                desc: format!("Installs CLI binary into PATH ({bin_name})"),
                                source_mech: source_mech.clone(),
                                raw: format!("binary: {bin_name}"),
                            });
                        }
                        "preflight" | "postflight" => {
                            permissions.push(Perm {
                                cat: PermCat::System,
                                desc: format!("Executes {key} hook during installation"),
                                source_mech: source_mech.clone(),
                                raw: key.clone(),
                            });
                        }
                        "uninstall" => {
                            if let Some(un_items) = val.as_array() {
                                for un in un_items {
                                    if let Some(un_obj) = un.as_object() {
                                        if un_obj.contains_key("launchctl") {
                                            permissions.push(Perm {
                                                cat: PermCat::System,
                                                desc: "Controls launchctl background services".to_string(),
                                                source_mech: source_mech.clone(),
                                                raw: "uninstall: launchctl".to_string(),
                                            });
                                        }
                                        if un_obj.contains_key("kext") {
                                            permissions.push(Perm {
                                                cat: PermCat::Hardware,
                                                desc: "Controls Kernel Extensions (kext)".to_string(),
                                                source_mech: source_mech.clone(),
                                                raw: "uninstall: kext".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        "zap" => {
                            if let Some(zap_items) = val.as_array() {
                                for zap in zap_items {
                                    if let Some(zap_obj) = zap.as_object() {
                                        if let Some(trash_arr) = zap_obj.get("trash").and_then(|t| t.as_array()) {
                                            for path_val in trash_arr {
                                                if let Some(path_str) = path_val.as_str() {
                                                    if let Some((cat, desc)) = describe_zap_path(path_str) {
                                                        permissions.push(Perm {
                                                            cat,
                                                            desc,
                                                            source_mech: source_mech.clone(),
                                                            raw: format!("zap: {path_str}"),
                                                        });
                                                    }
                                                }
                                            }
                                        } else if let Some(path_str) = zap_obj.get("trash").and_then(|t| t.as_str()) {
                                            if let Some((cat, desc)) = describe_zap_path(path_str) {
                                                permissions.push(Perm {
                                                    cat,
                                                    desc,
                                                    source_mech: source_mech.clone(),
                                                    raw: format!("zap: {path_str}"),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Caveats analysis
    if let Some(caveats_str) = cask.get("caveats").and_then(|v| v.as_str()) {
        permissions.extend(analyze_caveats(caveats_str, &source_mech));
    }

    let mut profile = AppProf::new(token);
    profile.permissions = permissions;
    Some(profile)
}

fn parse_formula(formula: &JsonValue) -> Option<AppProf> {
    let name = formula.get("name")?.as_str()?.to_string();
    let mut permissions = Vec::new();
    let source_mech = "homebrew-formula".to_string();

    // Background Service
    if let Some(service) = formula.get("service") {
        if let Some(svc_obj) = service.as_object() {
            let req_root = svc_obj.get("require_root").and_then(|v| v.as_bool()).unwrap_or(false);
            if req_root {
                permissions.push(Perm {
                    cat: PermCat::System,
                    desc: "Installs a root background daemon (requires root privileges / LaunchDaemon)".to_string(),
                    source_mech: source_mech.clone(),
                    raw: "service: require_root=true".to_string(),
                });
            } else {
                permissions.push(Perm {
                    cat: PermCat::System,
                    desc: "Installs a background service (LaunchAgent)".to_string(),
                    source_mech: source_mech.clone(),
                    raw: "service".to_string(),
                });
            }
        }
    }

    // Post-install steps
    let post_install_defined = formula.get("post_install_defined").and_then(|v| v.as_bool()).unwrap_or(false);
    let has_post_install_steps = formula.get("post_install_steps")
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);

    if post_install_defined || has_post_install_steps {
        permissions.push(Perm {
            cat: PermCat::System,
            desc: "Executes post-install configuration scripts".to_string(),
            source_mech: source_mech.clone(),
            raw: "post_install".to_string(),
        });
    }

    // Dependencies
    if let Some(deps_arr) = formula.get("dependencies").and_then(|v| v.as_array()) {
        for dep_val in deps_arr {
            if let Some(dep_name) = dep_val.as_str() {
                for (needle, desc) in PRIVILEGED_DEPS {
                    if dep_name == *needle {
                        permissions.push(Perm {
                            cat: PermCat::System,
                            desc: desc.to_string(),
                            source_mech: "homebrew-deps".to_string(),
                            raw: format!("dependencies: {dep_name}"),
                        });
                    }
                }
            }
        }
    }

    // Keg-only (can shadow system binaries or requires custom linking)
    let keg_only = formula.get("keg_only").and_then(|v| v.as_bool()).unwrap_or(false);
    if keg_only {
        let reason = formula.get("keg_only_reason")
            .and_then(|v| v.get("explanation"))
            .and_then(|v| v.as_str())
            .unwrap_or("Shadows system software or conflicts with macOS built-in tools");
        permissions.push(Perm {
            cat: PermCat::System,
            desc: format!("Keg-only formula: {reason}"),
            source_mech: source_mech.clone(),
            raw: format!("keg_only: {reason}"),
        });
    }

    // Caveats analysis
    if let Some(caveats_str) = formula.get("caveats").and_then(|v| v.as_str()) {
        permissions.extend(analyze_caveats(caveats_str, &source_mech));
    }

    let mut profile = AppProf::new(name);
    profile.permissions = permissions;
    Some(profile)
}

fn fetch_installed_json() -> Result<String, CollectorError> {
    let output = Command::new("brew")
        .arg("info")
        .arg("--json=v2")
        .arg("--installed")
        .output()
        .map_err(|_| CollectorError::NotInst("brew".to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(CollectorError::CmdErr(format!("brew info --json=v2 failed: {stderr}")));
    }

    let text = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(text)
}

pub fn collect() -> Result<Vec<AppProf>, String> {
    let json_text = fetch_installed_json().map_err(|e| e.to_string())?;
    let root = parse_json(&json_text).map_err(|e| format!("Failed to parse brew JSON: {e}"))?;

    let mut profiles = Vec::new();

    // Parse formulae
    if let Some(formulae) = root.get("formulae").and_then(|v| v.as_array()) {
        for f in formulae {
            if let Some(prof) = parse_formula(f) {
                profiles.push(prof);
            }
        }
    }

    // Parse casks
    if let Some(casks) = root.get("casks").and_then(|v| v.as_array()) {
        for c in casks {
            if let Some(prof) = parse_cask(c) {
                profiles.push(prof);
            }
        }
    }

    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parser_basic() {
        let json = r#"{
            "name": "tailscale",
            "count": 42,
            "active": true,
            "empty": null,
            "tags": ["vpn", "networking"],
            "nested": { "key": "value\nwith \"quotes\"" }
        }"#;

        let parsed = parse_json(json).expect("Should parse json");
        assert_eq!(parsed.get("name").and_then(|v| v.as_str()), Some("tailscale"));
        assert_eq!(parsed.get("active").and_then(|v| v.as_bool()), Some(true));
        assert!(parsed.get("empty").unwrap() == &JsonValue::Null);

        let tags = parsed.get("tags").and_then(|v| v.as_array()).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].as_str(), Some("vpn"));
        assert_eq!(tags[1].as_str(), Some("networking"));

        let nested = parsed.get("nested").and_then(|v| v.get("key")).and_then(|v| v.as_str()).unwrap();
        assert_eq!(nested, "value\nwith \"quotes\"");
    }

    #[test]
    fn test_parse_formula_service() {
        let json_str = r#"{
            "name": "tailscale",
            "service": {
                "run": "/usr/local/bin/tailscaled",
                "require_root": true
            },
            "dependencies": ["dbus"]
        }"#;

        let val = parse_json(json_str).unwrap();
        let prof = parse_formula(&val).expect("Should parse formula");
        assert_eq!(prof.app_id, "tailscale");
        assert!(prof.permissions.iter().any(|p| p.desc.contains("root background daemon")));
        assert!(prof.permissions.iter().any(|p| p.desc.contains("Depends on dbus")));
    }

    #[test]
    fn test_parse_cask_artifacts() {
        let json_str = r#"{
            "token": "aerospace",
            "artifacts": [
                { "app": ["AeroSpace.app"] },
                { "binary": ["aerospace"] },
                { "zap": [{ "trash": ["~/Library/Preferences/aerospace.plist"] }] }
            ],
            "caveats": "Requires Accessibility permissions in System Settings"
        }"#;

        let val = parse_json(json_str).unwrap();
        let prof = parse_cask(&val).expect("Should parse cask");
        assert_eq!(prof.app_id, "aerospace");
        assert!(prof.permissions.iter().any(|p| p.desc.contains("GUI Application bundle")));
        assert!(prof.permissions.iter().any(|p| p.desc.contains("Accessibility permissions")));
        assert!(prof.permissions.iter().any(|p| p.desc.contains("Preferences directory")));
    }

    #[test]
    fn test_collect_live() {
        if Command::new("which").arg("brew").output().map(|o| o.status.success()).unwrap_or(false) {
            let result = collect();
            assert!(result.is_ok(), "collect() failed: {:?}", result.err());
            let profiles = result.unwrap();
            assert!(!profiles.is_empty(), "Profiles should not be empty on system with brew");
        }
    }
}
