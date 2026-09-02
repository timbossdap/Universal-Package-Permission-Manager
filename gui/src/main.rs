use core::AppProf;
use qmetaobject::qtcore::core_application::QCoreApplication;
use qmetaobject::*;
use std::cell::RefCell;
use std::process::Command;

// which package source the gui is currently showing
#[derive(Clone, Copy, PartialEq)]
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

fn source_label(src: Source) -> &'static str {
    match src {
        Source::Flatpak => "flatpak",
        Source::Pacman => "pacman",
        Source::Homebrew => "homebrew",
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

    // true while the *currently selected* source is still being collected.
    // this used to only ever be true once at startup - now that sources can
    // be (re)loaded lazily, it flips back to true whenever the user selects
    // a tab whose data hasn't been fetched yet
    loading: qt_property!(bool; NOTIFY loading_changed),
    loading_changed: qt_signal!(),

    // mirrors the persisted preference (see load_prefs/save_prefs below) -
    // read once at startup to decide whether main() does a live scan or
    // loads the cache, then kept here just so the settings page has
    // something to bind its Switch to and reflect back what's actually in
    // effect for *this* run
    auto_refresh_on_launch: qt_property!(bool; NOTIFY auto_refresh_on_launch_changed),
    auto_refresh_on_launch_changed: qt_signal!(),

    // when true, only Flatpak is scanned automatically at launch - Pacman
    // and Homebrew are deferred until the user actually clicks their tab.
    // Flatpak is exempt on purpose: it's always eager, so there's
    // something on screen the moment the window appears
    lazy_load_tabs: qt_property!(bool; NOTIFY lazy_load_tabs_changed),
    lazy_load_tabs_changed: qt_signal!(),

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

            let already_loaded = self.is_current_source_loaded();
            self.loading = !already_loaded;
            self.loading_changed();

            self.refresh_app_ids();
            self.permissions = QVariantList::default();
            self.permissions_changed();
            self.permissions_hi_risk = QVariantList::default();
            self.permissions_hi_risk_changed();

            if self.lazy_load_tabs && !already_loaded {
                self.ensure_source_loading();
            }
        }
    ),

    // kicks off a background scan for whichever source is currently
    // selected, if (and only if) it isn't already loaded or already in
    // flight. only meaningful for Pacman/Homebrew - Flatpak is always
    // started eagerly in main() so there's nothing for this to do there
    ensure_source_loading: qt_method!(
        fn ensure_source_loading(&mut self) {
            let src = self.current_source;

            if src == Source::Flatpak {
                return;
            }

            let already_running = match src {
                Source::Pacman => self.pacman_loading_flag,
                Source::Homebrew => self.homebrew_loading_flag,
                Source::Flatpak => true,
            };
            if already_running {
                return;
            }

            match src {
                Source::Pacman => self.pacman_loading_flag = true,
                Source::Homebrew => self.homebrew_loading_flag = true,
                Source::Flatpak => {}
            }

            let handle = match self.self_handle {
                Some(h) => h,
                None => return,
            };
            let auto_refresh = self.auto_refresh_on_launch;

            // built here, on the gui thread, then handed off to the worker
            // thread below - same pattern main() uses for the eager scans
            let apply = qmetaobject::queued_callback(move |profiles: Vec<AppProf>| {
                let mut b = handle.borrow_mut();
                match src {
                    Source::Pacman => {
                        b.pacman_profiles = profiles;
                        b.pacman_loaded = true;
                        b.pacman_loading_flag = false;
                    }
                    Source::Homebrew => {
                        b.homebrew_profiles = profiles;
                        b.homebrew_loaded = true;
                        b.homebrew_loading_flag = false;
                    }
                    Source::Flatpak => {}
                }
                if b.current_source == src {
                    b.refresh_app_ids();
                    b.loading = false;
                    b.loading_changed();
                }
            });

            std::thread::spawn(move || {
                let profiles = collect_and_cache(src, auto_refresh);
                apply(profiles);
            });
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
            save_prefs(&Prefs {
                auto_refresh_on_launch: val,
                lazy_load_tabs: self.lazy_load_tabs,
            });
        }
    ),

    // called from the settings page when "Load tabs on demand" is
    // toggled. like auto-refresh, this only affects *future* launches -
    // it doesn't retroactively cancel or start scans for the current run
    set_lazy_load_tabs: qt_method!(
        fn set_lazy_load_tabs(&mut self, val: bool) {
            self.lazy_load_tabs = val;
            self.lazy_load_tabs_changed();
            save_prefs(&Prefs {
                auto_refresh_on_launch: self.auto_refresh_on_launch,
                lazy_load_tabs: val,
            });
        }
    ),

    flatpak_profiles: Vec<AppProf>,
    pacman_profiles: Vec<AppProf>,
    homebrew_profiles: Vec<AppProf>,
    current_source: Source,

    // per-source "has this ever finished a scan this run" flags
    flatpak_loaded: bool,
    pacman_loaded: bool,
    homebrew_loaded: bool,

    // per-source "is a background scan currently in flight" flags - guards
    // against double-spawning a scan if the user bounces between tabs
    // while one is still loading. flatpak has no such flag since it's
    // never triggered lazily
    pacman_loading_flag: bool,
    homebrew_loading_flag: bool,

    // set once, right after the bridge is created in main(), so methods
    // triggered from qml (like ensure_source_loading) can spawn a worker
    // thread and hand it a way to write back into this same bridge later
    self_handle: Option<&'static RefCell<UppmBridge>>,
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

    fn is_current_source_loaded(&self) -> bool {
        match self.current_source {
            Source::Flatpak => self.flatpak_loaded,
            Source::Pacman => self.pacman_loaded,
            Source::Homebrew => self.homebrew_loaded,
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

// --- persisted preferences: "auto-refresh on launch" + "load tabs on
// demand" ---------------------------------------------------------------
//
// deliberately a flat "key=value" text file rather than pulling in a
// config-file crate for two booleans - std::fs is all this needs.

struct Prefs {
    auto_refresh_on_launch: bool,
    lazy_load_tabs: bool,
}

fn prefs_path() -> std::path::PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home).join(".config")
        });
    base.join("uppm").join("prefs.txt")
}

// defaults match the app's original behaviour (auto-refresh on, lazy
// loading off) whenever the prefs file is missing, unreadable, or doesn't
// explicitly say otherwise
fn load_prefs() -> Prefs {
    let mut prefs = Prefs {
        auto_refresh_on_launch: true,
        lazy_load_tabs: false,
    };

    if let Ok(text) = std::fs::read_to_string(prefs_path()) {
        for line in text.lines() {
            match line.trim() {
                "auto_refresh=false" => prefs.auto_refresh_on_launch = false,
                "auto_refresh=true" => prefs.auto_refresh_on_launch = true,
                "lazy_load_tabs=true" => prefs.lazy_load_tabs = true,
                "lazy_load_tabs=false" => prefs.lazy_load_tabs = false,
                _ => {}
            }
        }
    }

    prefs
}

fn save_prefs(prefs: &Prefs) {
    let path = prefs_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let text = format!(
        "auto_refresh={}\nlazy_load_tabs={}\n",
        prefs.auto_refresh_on_launch, prefs.lazy_load_tabs
    );
    let _ = std::fs::write(path, text);
}

// --- scan cache -----------------------------------------------------------
//
// plain tab-separated text, one file per source ("scan_cache_flatpak.txt",
// "scan_cache_pacman.txt", "scan_cache_homebrew.txt"), each holding
// "APP"/"PERM" lines - no serde dependency needed for two simple record
// kinds. splitting the cache per source (rather than one combined file)
// is what lets each source be scanned and cached completely independently
// of the others. tabs/newlines inside any field get flattened to spaces
// on the way out so the line format can't be broken by unusual package
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

fn cache_dir() -> std::path::PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(home).join(".cache")
        });
    base.join("uppm")
}

fn cache_path(src: Source) -> std::path::PathBuf {
    cache_dir().join(format!("scan_cache_{}.txt", source_label(src)))
}

fn save_cache(path: &std::path::Path, profiles: &[AppProf]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut out = String::new();
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
    std::fs::write(path, out)
}

fn load_cache(path: &std::path::Path) -> std::io::Result<Vec<AppProf>> {
    let text = std::fs::read_to_string(path)?;
    let mut profiles: Vec<AppProf> = Vec::new();

    for line in text.lines() {
        let mut parts = line.splitn(5, '\t');
        match parts.next() {
            Some("APP") => {
                let app_id = parts.next().unwrap_or("").to_string();
                profiles.push(AppProf::new(app_id));
            }
            Some("PERM") => {
                let cat = perm_cat_from_str(parts.next().unwrap_or(""));
                let desc = parts.next().unwrap_or("").to_string();
                let raw = parts.next().unwrap_or("").to_string();
                let source_mech = parts.next().unwrap_or("").to_string();
                if let Some(app) = profiles.last_mut() {
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

    Ok(profiles)
}

fn collect_live(src: Source) -> Vec<AppProf> {
    match src {
        Source::Flatpak => core::collectors::flatpak::collect().unwrap_or_default(),
        Source::Pacman => core::collectors::pacman::collect().unwrap_or_default(),
        Source::Homebrew => core::collectors::homebrew::collect().unwrap_or_default(),
    }
}

// scans a single source live, or falls back to its own cache file when
// "auto-refresh on launch" is off. this is the one function both the
// eager startup scans (in main()) and the on-demand lazy scans
// (ensure_source_loading) call, so both paths behave identically
fn collect_and_cache(src: Source, auto_refresh: bool) -> Vec<AppProf> {
    let path = cache_path(src);

    if !auto_refresh {
        if let Ok(profiles) = load_cache(&path) {
            if !profiles.is_empty() {
                return profiles;
            }
        }
    }

    let profiles = collect_live(src);
    let _ = save_cache(&path, &profiles);
    profiles
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

    // Qt.labs.settings' Settings type (used by AppSettings.qml) refuses to
    // read/write anything and logs "Failed to initialize QSettings instance"
    // until an organization + application name are set on QCoreApplication -
    // this has to happen before QmlEngine::new() spins up the app, same as
    // the style var above
    QCoreApplication::set_organization_name("uppm".into());
    QCoreApplication::set_application_name("UPPM".into());

    let prefs = load_prefs();
    let auto_refresh_on_launch = prefs.auto_refresh_on_launch;
    let lazy_load_tabs = prefs.lazy_load_tabs;
    let available_sources = detect_available_sources();
    let all_sources = all_sources();

    // leaked on purpose: this bridge is meant to live for the whole program,
    // so a plain 'static reference is simpler here than fighting Rc/lifetimes
    let bridge: &'static RefCell<UppmBridge> = Box::leak(Box::new(RefCell::new(UppmBridge {
        loading: true,
        auto_refresh_on_launch,
        lazy_load_tabs,
        available_sources,
        all_sources,
        ..Default::default()
    })));

    // give the bridge a way to reach itself later, so methods invoked from
    // qml (ensure_source_loading) can spawn worker threads that write back
    // into it once a lazily-triggered scan finishes
    bridge.borrow_mut().self_handle = Some(bridge);

    let mut engine = QmlEngine::new();
    engine.set_object_property("uppm".into(), unsafe { QObjectPinned::new(bridge) });

    // Flatpak always scans immediately on launch, regardless of the
    // "load tabs on demand" setting, so there's something on screen the
    // moment the window appears.
    {
        let apply = qmetaobject::queued_callback(move |profiles: Vec<AppProf>| {
            let mut b = bridge.borrow_mut();
            b.flatpak_profiles = profiles;
            b.flatpak_loaded = true;
            if b.current_source == Source::Flatpak {
                b.refresh_app_ids();
                b.loading = false;
                b.loading_changed();
            }
        });
        std::thread::spawn(move || {
            let profiles = collect_and_cache(Source::Flatpak, auto_refresh_on_launch);
            apply(profiles);
        });
    }

    // Pacman and Homebrew: scanned eagerly too unless "load tabs on
    // demand" is turned on, in which case select_source()/
    // ensure_source_loading() kick each of them off the first time their
    // tab is actually opened. each source gets its own thread + callback
    // so one slow collector can never block another source's list (or
    // Flatpak's) from showing up - they were previously bundled into a
    // single thread/callback, which meant Flatpak's list couldn't appear
    // until Pacman and Homebrew were both done too.
    if !lazy_load_tabs {
        for src in [Source::Pacman, Source::Homebrew] {
            let apply = qmetaobject::queued_callback(move |profiles: Vec<AppProf>| {
                let mut b = bridge.borrow_mut();
                match src {
                    Source::Pacman => {
                        b.pacman_profiles = profiles;
                        b.pacman_loaded = true;
                    }
                    Source::Homebrew => {
                        b.homebrew_profiles = profiles;
                        b.homebrew_loaded = true;
                    }
                    Source::Flatpak => {}
                }
                if b.current_source == src {
                    b.refresh_app_ids();
                    b.loading = false;
                    b.loading_changed();
                }
            });
            std::thread::spawn(move || {
                let profiles = collect_and_cache(src, auto_refresh_on_launch);
                apply(profiles);
            });
        }
    }

    engine.load_file("./gui/qml/main.qml".into());
    engine.exec();
}
