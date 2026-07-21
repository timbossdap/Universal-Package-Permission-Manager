pub mod collectors;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PermCat {
    Filesystem,
    Network,
    System,
    Desktop,
    Hardware,
}

impl std::fmt::Display for PermCat {
    fn fmt(&self, f: &mut std::fmt::_Formatter) -> std::fmt::Result {
        let label = match self {
            PermCat::Filesystem => "Filesystem",
            PermCat::Network => "Network",
            PermCat::System => "System",
            PermCat::Desktop => "Desktop",
            PermCat::Hardware => "Hardware",
        };
        write!(f, "{}", label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Perm {
    pub cat: PermCat,
    pub desc: String,
    pub raw: String,
    pub source_mech: String,
}

impl std::fmt::Display for Perm {
    fn fmt(&self, f: &mut std::fmt::_Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.cat, self.desc)
    }
}

impl Perm {
    pub fn is_hi_risk(&self) -> bool {
        if self.desc.contains("system")
            || self.desc.contains("desktop")
            || self.desc.contains("hardware")
            || self.desc.contains("filesystem")
        {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppProf {
    pub app_id: String,
    pub permissions: Vec<Perm>,
}

impl AppProf {
    pub fn new(app_id: String) -> AppProf {
        AppProf {
            app_id,
            permissions: Vec::new(),
        }
    }

    pub fn flagged_count(&self) -> usize {
        self.permissions.iter().filter(|p| p.is_hi_risk()).count()
    }
}

pub struct ScanSum {
    pub app_count: u32,
    pub flagged_count: u32,
}

impl ScanSum {
    pub fn new(app_count: u32, flagged_count: u32) -> ScanSum {
        ScanSum {
            app_count,
            flagged_count,
        }
    }

    pub fn from_profiles(profiles: &[AppProf]) -> ScanSum {
        let flagged: usize = profiles.iter().map(|p| p.flagged_count()).sum();
        ScanSum::new(profiles.len() as u32, flagged as u32)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CollectorError {
    NotInst(String),
    CmdErr(String),
}

impl std::fmt::Display for CollectorError {
    fn fmt(&self, f: &mut std::fmt::_Formatter) -> std::fmt::Result {
        match self {
            CollectorError::NotInst(msg) => write!(f, "not installed: {}", msg),
            CollectorError::CmdErr(msg) => write!(f, "command failed: {}", msg),
        }
    }
}

impl std::error::Error for CollectorError {}
