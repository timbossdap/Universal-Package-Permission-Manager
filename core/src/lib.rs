//! Core types shared by all the collectors. Started out flatpak-only, the
//! idea is snap/native distro packages get their own collector later and
//! just plug into the same AppProfile/Permission shapes.

pub mod collectors;

/// Broad buckets we sort permissions into. Roughly mirrors how flatpak
/// groups things in `flatpak info --show-permissions`, kept generic enough
/// that other collectors can reuse it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PermissionCategory {
    Filesystem,
    Hardware,
    Network,
    Desktop,
    System,
}

impl std::fmt::Display for PermissionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let label = match self {
            PermissionCategory::Filesystem => "Filesystem",
            PermissionCategory::Hardware => "Hardware",
            PermissionCategory::Network => "Network",
            PermissionCategory::Desktop => "Desktop",
            PermissionCategory::System => "System",
        };
        write!(f, "{}", label)
    }
}

/// One permission grant found for an app, plus enough context to explain
/// why it got flagged and where it came from.
#[derive(Debug, Clone)]
pub struct Permission {
    pub category: PermissionCategory,
    pub description: String,
    pub source_mechanism: String,
    /// Raw override string as the source tool reported it, unparsed.
    /// Kept around for debugging when the description doesn't add up.
    pub raw: String,
}

impl Permission {
    /// Rough "worth a second look" check, not a real audit - just enough
    /// to separate the noisy stuff (GPU, wayland) from things like full
    /// filesystem or all-devices access.
    pub fn is_high_risk(&self) -> bool {
        let d = self.description.to_lowercase();
        d.contains("entire filesystem")
            || d.contains("all hardware devices")
            || (self.category == PermissionCategory::Filesystem && d.contains("read-write"))
    }
}

/// Everything we know about one installed app.
#[derive(Debug, Clone)]
pub struct AppProfile {
    pub app_id: String,
    pub permissions: Vec<Permission>,
}

impl AppProfile {
    pub fn new(app_id: String) -> AppProfile {
        AppProfile {
            app_id,
            permissions: Vec::new(),
        }
    }

    /// How many of this app's permissions tripped is_high_risk().
    pub fn flagged_count(&self) -> usize {
        self.permissions.iter().filter(|p| p.is_high_risk()).count()
    }
}

/// Totals across a whole scan, mostly so the CLI has something to print at
/// the end instead of just a wall of per-app output.
pub struct ScanSummary {
    pub app_count: u32,
    pub flagged_count: u32,
}

impl ScanSummary {
    pub fn new(app_count: u32, flagged_count: u32) -> ScanSummary {
        ScanSummary { app_count, flagged_count }
    }

    /// Build a summary straight from scan results instead of counting by
    /// hand in the caller.
    pub fn from_profiles(profiles: &[AppProfile]) -> ScanSummary {
        let flagged: usize = profiles.iter().map(|p| p.flagged_count()).sum();
        ScanSummary::new(profiles.len() as u32, flagged as u32)
    }
}

/// Errors a collector can hit while gathering permission data. An enum
/// instead of raw strings so callers can actually branch on what went
/// wrong (missing binary vs bad output) instead of just printing text.
#[derive(Debug)]
pub enum CollectorError {
    /// The backing tool (flatpak, snap, etc) isn't installed or on PATH.
    NotInstalled(String),
    /// The tool ran but exited non-zero.
    CommandFailed(String),
    /// Got output back but couldn't make sense of it.
    Malformed(String),
}

impl std::fmt::Display for CollectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CollectorError::NotInstalled(msg) => write!(f, "not installed: {}", msg),
            CollectorError::CommandFailed(msg) => write!(f, "command failed: {}", msg),
            CollectorError::Malformed(msg) => write!(f, "couldn't parse output: {}", msg),
        }
    }
}

impl std::error::Error for CollectorError {}
