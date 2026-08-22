use core::AppProf;
use qmetaobject::*;
use std::cell::RefCell;

// which package source the gui is currently showing
// this is the enum from lesson 4 - two variants, nothing else possible
#[derive(Clone, Copy)]
enum Source {
    Flatpak,
    Pacman,
}

impl Default for Source {
    fn default() -> Self {
        Source::Flatpak
    }
}

#[derive(QObject, Default)]
struct UppmBridge {
    base: qt_base_class!(trait QObject),
    app_ids: qt_property!(QVariantList; NOTIFY app_ids_changed),
    app_ids_changed: qt_signal!(),
    permissions: qt_property!(QVariantList; NOTIFY permissions_changed),
    permissions_changed: qt_signal!(),

    // called from qml when the tab bar changes: 0 = flatpak, 1 = pacman
    select_source: qt_method!(
        fn select_source(&mut self, tab_index: i32) {
            self.current_source = match tab_index {
                0 => Source::Flatpak,
                1 => Source::Pacman,
                _ => Source::Flatpak,
            };
            self.refresh_app_ids();
            self.permissions = QVariantList::default();
            self.permissions_changed();
        }
    ),

    select_app: qt_method!(
        fn select_app(&mut self, index: i32) {
            // build the perms list fully here first - this borrows self
            // immutably (through current_profiles) but that borrow ends
            // as soon as .map() finishes, before we touch self.permissions below
            let perms = self
                .current_profiles()
                .get(index as usize)
                .map(|profile| {
                    profile
                        .permissions
                        .iter()
                        .map(|p| QString::from(format!("{}", p)).to_qvariant())
                        .collect::<QVariantList>()
                });

            if let Some(perms) = perms {
                self.permissions = perms;
                self.permissions_changed();
            }
        }
    ),

    flatpak_profiles: Vec<AppProf>,
    pacman_profiles: Vec<AppProf>,
    current_source: Source,
}

impl UppmBridge {
    // match picks which vec to hand back based on the enum - this is the
    // one place that knows about both sources, everything else just calls this
    fn current_profiles(&self) -> &Vec<AppProf> {
        match self.current_source {
            Source::Flatpak => &self.flatpak_profiles,
            Source::Pacman => &self.pacman_profiles,
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

    fn from_profiles(flatpak_profiles: Vec<AppProf>, pacman_profiles: Vec<AppProf>) -> Self {
        let mut bridge = UppmBridge {
            flatpak_profiles,
            pacman_profiles,
            ..Default::default()
        };
        bridge.refresh_app_ids();
        bridge
    }
}

fn main() {
    let flatpak_profiles = core::collectors::flatpak::collect().unwrap_or_default();
    let pacman_profiles = core::collectors::pacman::collect().unwrap_or_default();
    let bridge = RefCell::new(UppmBridge::from_profiles(flatpak_profiles, pacman_profiles));

    let mut engine = QmlEngine::new();
    engine.set_object_property("uppm".into(), unsafe { QObjectPinned::new(&bridge) });
    engine.load_file("./gui/qml/main.qml".into());
    engine.exec();
}
