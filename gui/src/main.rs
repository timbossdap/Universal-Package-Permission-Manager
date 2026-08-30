use core::AppProf;
use qmetaobject::*;
use std::cell::RefCell;
use std::process::Command;

// which package source the gui is currently showing
#[derive(Clone, Copy)]
enum Source {
    Flatpak,
    Pacman,
    Homebrew,
}

impl Default for Source {
    fn default() -> Self {
        Source::Flatpak
    }
}

fn is_installed(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn detect_available_sources() -> QVariantList {
    let mut sources: QVariantList = QVariantList::default();
    if is_installed("flatpak") {
        sources.push(QString::from("Flatpak").to_qvariant());
    }
    if is_installed("pacman") {
        sources.push(QString::from("Pacman").to_qvariant());
    }
    if is_installed("brew") {
        sources.push(QString::from("Homebrew").to_qvariant());
    }
    sources
}

fn all_sources() -> QVariantList {
    let mut sources: QVariantList = QVariantList::default();
    sources.push(QString::from("Flatpak").to_qvariant());
    sources.push(QString::from("Pacman").to_qvariant());
    sources.push(QString::from("Homebrew").to_qvariant());
    sources
}

#[derive(QObject, Default)]
struct UppmBridge {
    base: qt_base_class!(trait QObject),
    app_ids: qt_property!(QVariantList; NOTIFY app_ids_changed),
    app_ids_changed: qt_signal!(),
    permissions: qt_property!(QVariantList; NOTIFY permissions_changed),
    permissions_changed: qt_signal!(),

    // parallel to `permissions` (same index = same permission) - true where
    // that permission's Perm::is_hi_risk() says so. kept as its own list
    // instead of baking a marker into the display string so QML can decide
    // how to render the flag (color, icon, etc.) without string-parsing
    permissions_hi_risk: qt_property!(QVariantList; NOTIFY permissions_hi_risk_changed),
    permissions_hi_risk_changed: qt_signal!(),

    // true while the collectors are still running in the background
    loading: qt_property!(bool; NOTIFY loading_changed),
    loading_changed: qt_signal!(),

    // mirrors the persisted preference (see load_pref_auto_refresh/
    // save_pref_auto_refresh below) - read once at startup to decide
    // whether main() does a live scan or loads the cache, then kept here
    // just so the settings page has something to bind its Switch to and
    // reflect back what's actually in effect for *this* run
    auto_refresh_on_launch: qt_property!(bool; NOTIFY auto_refresh_on_launch_changed),
    auto_refresh_on_launch_changed: qt_signal!(),

    // list of all supported package manager sources (Flatpak, Pacman, Homebrew)
    all_sources: qt_property!(QVariantList; NOTIFY all_sources_changed),
    all_sources_changed: qt_signal!(),

    // list of source names available on this system, e.g. ["Flatpak", "Pacman"]
    // QML uses this to build the tab bar dynamically - only installed sources show up
    available_sources: qt_property!(QVariantList; NOTIFY available_sources_changed),
    available_sources_changed: qt_signal!(),

    // called from qml when a tab is selected - receives the source name string
    // (e.g. "Flatpak", "Pacman", or "Homebrew") instead of a numeric index, so the mapping
    // stays correct regardless of which tabs are visible
    select_source: qt_method!(
        fn select_source(&mut self, source_name: String) {
            self.current_source = match source_name.as_str() {
                "Flatpak" => Source::Flatpak,
                "Pacman" => Source::Pacman,
                "Homebrew" => Source::Homebrew,
                _ => Source::Flatpak,
            };
            self.refresh_app_ids();
            self.permissions = QVariantList::default();
            self.permissions_changed();
            self.permissions_hi_risk = QVariantList::default();
            self.permissions_hi_risk_changed();
        }
    ),

    select_app: qt_method!(
        fn select_app(&mut self, index: i32) {
            // build both lists fully here first - this borrows self
            // immutably (through current_profiles) but that borrow ends
            // as soon as .map() finishes, before we touch self.permissions below
            let perms = self.current_profiles().get(index as usize).map(|profile| {
                let texts: QVariantList = profile
                    .permissions
                    .iter()
                    .map(|p| QString::from(format!("{}", p)).to_qvariant())
                    .collect();
                let hi_risk: QVariantList = profile
                    .permissions
                    .iter()
                    .map(|p| p.is_hi_risk().to_qvariant())
                    .collect();
                (texts, hi_risk)
            });

            if let Some((texts, hi_risk)) = perms {
                self.permissions = texts;
                self.permissions_changed();
                self.permissions_hi_risk = hi_risk;
                self.permissions_hi_risk_changed();
            }
        }
    ),

    // called from the settings page when the "Auto-refresh on launch"
    // switch is toggled. only persists the preference for *next* launch -
    // it can't retroactively change how the data already on screen was
    // obtained this run, same as the setting's description promises
    set_auto_refresh_on_launch: qt_method!(
        fn set_auto_refresh_on_launch(&mut self, val: bool) {
            self.auto_refresh_on_launch = val;
            self.auto_refresh_on_launch_changed();
            save_pref_auto_refresh(val);
        }
    ),

    flatpak_profiles: Vec<AppProf>,
    pacman_profiles: Vec<AppProf>,
    homebrew_profiles: Vec<AppProf>,
    current_source: Source,
}

impl UppmBridge {
    // match picks which vec to hand back based on the enum - this is the
    // one place that knows about all sources, everything else just calls this
    fn current_profiles(&self) -> &Vec<AppProf> {
        match self.current_source {
            Source::Flatpak => &self.flatpak_profiles,
            Source::Pacman => &self.pacman_profiles,
            Source::Homebrew => &self.homebrew_profiles,
        }
    }

    fn refresh_app_ids(&mut self) {
        let ids: QVariantList = self
            .current_profiles()
            .iter()
            .map(|p| QString::from(p.app_id.clone()).to_qvariant())
            .collect();
        self.app_ids = ids;
        self.app_ids_changed();
    }
}

// --- persisted preference: "auto-refresh on launch" ---------------------
//
// deliberately a flat "key=value" text file rather than pulling in a
// config-file crate for one boolean - std::fs is all this needs.

fn prefs_path() -> std::path::PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home).join(".config")
        });
    base.join("uppm").join("prefs.txt")
}

// defaults to true (matches the scan-every-launch behaviour the app has
// always had) whenever the prefs file is missing, unreadable, or doesn't
// explicitly say otherwise
fn load_pref_auto_refresh() -> bool {
    match std::fs::read_to_string(prefs_path()) {
        Ok(text) => text.trim() != "auto_refresh=false",
        Err(_) => true,
    }
}

fn save_pref_auto_refresh(val: bool) {
    let path = prefs_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, format!("auto_refresh={}", val));
}

// --- scan cache -----------------------------------------------------------
//
// plain tab-separated text, one "SOURCE" line per package source followed
// by its "APP"/"PERM" lines - no serde dependency needed for three simple
// record kinds. tabs/newlines inside any field get flattened to spaces on
// the way out so the line format can't be broken by unusual package
// metadata; the very rare loss of an embedded tab character in, say, a raw
// permission string is an acceptable trade-off for a cache file.

fn perm_cat_to_str(cat: &core::PermCat) -> &'static str {
    match cat {
        core::PermCat::Filesystem => "Filesystem",
        core::PermCat::Network => "Network",
        core::PermCat::System => "System",
        core::PermCat::Desktop => "Desktop",
        core::PermCat::Hardware => "Hardware",
    }
}

fn perm_cat_from_str(s: &str) -> core::PermCat {
    match s {
        "Filesystem" => core::PermCat::Filesystem,
        "Network" => core::PermCat::Network,
        "Desktop" => core::PermCat::Desktop,
        "Hardware" => core::PermCat::Hardware,
        _ => core::PermCat::System,
    }
}

fn sanitize_field(s: &str) -> String {
    s.replace('\t', " ").replace('\n', " ")
}

fn cache_path() -> std::path::PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home).join(".cache")
        });
    base.join("uppm").join("scan_cache.txt")
}

fn append_profiles(out: &mut String, source_label: &str, profiles: &[AppProf]) {
    out.push_str("SOURCE\t");
    out.push_str(source_label);
    out.push('\n');
    for app in profiles {
        out.push_str("APP\t");
        out.push_str(&sanitize_field(&app.app_id));
        out.push('\n');
        for p in &app.permissions {
            out.push_str("PERM\t");
            out.push_str(perm_cat_to_str(&p.cat));
            out.push('\t');
            out.push_str(&sanitize_field(&p.desc));
            out.push('\t');
            out.push_str(&sanitize_field(&p.raw));
            out.push('\t');
            out.push_str(&sanitize_field(&p.source_mech));
            out.push('\n');
        }
    }
}

fn save_cache(
    path: &std::path::Path,
    flatpak: &[AppProf],
    pacman: &[AppProf],
    homebrew: &[AppProf],
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut out = String::new();
    append_profiles(&mut out, "flatpak", flatpak);
    append_profiles(&mut out, "pacman", pacman);
    append_profiles(&mut out, "homebrew", homebrew);
    std::fs::write(path, out)
}

fn load_cache(path: &std::path::Path) -> std::io::Result<(Vec<AppProf>, Vec<AppProf>, Vec<AppProf>)> {
    let text = std::fs::read_to_string(path)?;
    let mut flatpak: Vec<AppProf> = Vec::new();
    let mut pacman: Vec<AppProf> = Vec::new();
    let mut homebrew: Vec<AppProf> = Vec::new();
    let mut current_source = "flatpak";

    for line in text.lines() {
        let mut parts = line.splitn(5, '\t');
        match parts.next() {
            Some("SOURCE") => {
                current_source = match parts.next() {
                    Some("pacman") => "pacman",
                    Some("homebrew") => "homebrew",
                    _ => "flatpak",
                };
            }
            Some("APP") => {
                let app_id = parts.next().unwrap_or("").to_string();
                let list = match current_source {
                    "pacman" => &mut pacman,
                    "homebrew" => &mut homebrew,
                    _ => &mut flatpak,
                };
                list.push(AppProf::new(app_id));
            }
            Some("PERM") => {
                let cat = perm_cat_from_str(parts.next().unwrap_or(""));
                let desc = parts.next().unwrap_or("").to_string();
                let raw = parts.next().unwrap_or("").to_string();
                let source_mech = parts.next().unwrap_or("").to_string();
                let list = match current_source {
                    "pacman" => &mut pacman,
                    "homebrew" => &mut homebrew,
                    _ => &mut flatpak,
                };
                if let Some(app) = list.last_mut() {
                    app.permissions.push(core::Perm {
                        cat,
                        desc,
                        raw,
                        source_mech,
                    });
                }
            }
            _ => {}
        }
    }

    Ok((flatpak, pacman, homebrew))
}

fn main() {
    // has to be set before QmlEngine::new() spins up the Qt application,
    // this is what actually turns on the material look.
    // set_var is unsafe in this edition because it's process-global and
    // could race with another thread reading env vars - not a problem here
    // since this is the very first thing we do, before any threads exist
    unsafe {
        std::env::set_var("QT_QUICK_CONTROLS_STYLE", "Material");
    }

    let auto_refresh_on_launch = load_pref_auto_refresh();
    let available_sources = detect_available_sources();
    let all_sources = all_sources();

    // leaked on purpose: this bridge is meant to live for the whole program,
    // so a plain 'static reference is simpler here than fighting Rc/lifetimes
    let bridge: &'static RefCell<UppmBridge> = Box::leak(Box::new(RefCell::new(UppmBridge {
        loading: true,
        auto_refresh_on_launch,
        available_sources,
        all_sources,
        ..Default::default()
    })));

    let mut engine = QmlEngine::new();
    engine.set_object_property("uppm".into(), unsafe { QObjectPinned::new(bridge) });

    // wraps a closure so it's safe to call from another thread - when called,
    // it hops back onto this (the gui) thread to actually run the closure body
    let apply_results = qmetaobject::queued_callback(
        move |(flatpak_profiles, pacman_profiles, homebrew_profiles): (Vec<AppProf>, Vec<AppProf>, Vec<AppProf>)| {
            let mut b = bridge.borrow_mut();
            b.flatpak_profiles = flatpak_profiles;
            b.pacman_profiles = pacman_profiles;
            b.homebrew_profiles = homebrew_profiles;
            b.refresh_app_ids();
            b.loading = false;
            b.loading_changed();
        },
    );

    let cache_file = cache_path();

    if auto_refresh_on_launch {
        // unchanged from before: always scan live off the gui thread so the
        // window shows up instantly. also opportunistically writes the
        // results to the cache file so a future launch with the toggle off
        // has something to fall back on
        let cache_for_thread = cache_file.clone();
        std::thread::spawn(move || {
            let flatpak_profiles = core::collectors::flatpak::collect().unwrap_or_default();
            let pacman_profiles = core::collectors::pacman::collect().unwrap_or_default();
            let homebrew_profiles = core::collectors::homebrew::collect().unwrap_or_default();
            let _ = save_cache(&cache_for_thread, &flatpak_profiles, &pacman_profiles, &homebrew_profiles);
            apply_results((flatpak_profiles, pacman_profiles, homebrew_profiles));
        });
    } else {
        // toggle is off: try the last cached scan first instead of hitting
        // flatpak/pacman/homebrew again. falls back to a live scan (and populates the
        // cache for next time) if there's no cache yet or it's empty
        let mut used_cache = false;
        if let Ok((flatpak_profiles, pacman_profiles, homebrew_profiles)) = load_cache(&cache_file) {
            if !flatpak_profiles.is_empty() || !pacman_profiles.is_empty() || !homebrew_profiles.is_empty() {
                used_cache = true;
                apply_results((flatpak_profiles, pacman_profiles, homebrew_profiles));
            }
        }

        if !used_cache {
            let cache_for_thread = cache_file.clone();
            std::thread::spawn(move || {
                let flatpak_profiles = core::collectors::flatpak::collect().unwrap_or_default();
                let pacman_profiles = core::collectors::pacman::collect().unwrap_or_default();
                let homebrew_profiles = core::collectors::homebrew::collect().unwrap_or_default();
                let _ = save_cache(&cache_for_thread, &flatpak_profiles, &pacman_profiles, &homebrew_profiles);
                apply_results((flatpak_profiles, pacman_profiles, homebrew_profiles));
            });
        }
    }

    engine.load_file("./gui/qml/main.qml".into());
    engine.exec();
}
