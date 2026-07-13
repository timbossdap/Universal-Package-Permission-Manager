pub enum PermCat {
    Filesystem,
    Network,
    System,
    Desktop,
    Hardware,
}

impl std::fmt::Display for PermCat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

pub struct Perms {
    pub category: PermCat,
    pub description: String,
    pub source_mechanism: String,
    pub raw: String,
}
impl std::fmt::Display for Perms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.category, self.description)
    }
}

impl Perms {
    pub fn is_high_risk(&self) -> bool {
        // Convert to lowercase once so all subsequent checks can be case-insensitive
        let d = self.description.to_lowercase();

        d.contains("network")
            || d.contains("system")
            || d.contains("desktop")
            || d.contains("hardware")
            || d.contains("filesystem")
    }
}
