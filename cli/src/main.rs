use core::{Permission, PermissionCategory, AppProfile};

fn main() {
    let cam_permission = Permission {
        category: PermissionCategory::Hardware,
        description: String::from("Camera access"),
        source_mechanism: String::from("flatpak"),
        raw: String::from("--device=all"),
    };
    let microphone_permission = Permission {
        category: PermissionCategory::Hardware,
        description: String::from("Microphone access"),
        source_mechanism: String::from("flatpak"),
        raw: String::from("--device=all"),
    };
    let network_permission = Permission {
        category: PermissionCategory::Network,
        description: String::from("Network access"),
        source_mechanism: String::from("flatpak"),
        raw: String::from("--network=host"),
    };
    let all_permissions = vec![cam_permission, microphone_permission, network_permission];
    let firefox = AppProfile {
        app_id: String::from("org.mozilla.firefox"),
        permissions: all_permissions,
    };
    println!("{:?}", firefox);
}
