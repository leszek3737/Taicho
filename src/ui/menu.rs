use dioxus::desktop::muda::accelerator::{Accelerator, Code, Modifiers};
use dioxus::desktop::muda::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu};

#[allow(dead_code)]
pub fn build_menu() -> Menu {
    let menu = Menu::new();

    let app_menu = Submenu::new("Taicho", true);
    let _ = app_menu.append_items(&[
        &PredefinedMenuItem::about(
            None,
            Some(AboutMetadata {
                name: Some("Taicho".to_string()),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                copyright: Some("MIT License".to_string()),
                ..Default::default()
            }),
        ),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id(
            "taicho_settings",
            "Settings\u{2026}",
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::Comma)),
        ),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::quit(None),
    ]);

    let file_menu = Submenu::new("File", true);
    let _ = file_menu.append_items(&[
        &MenuItem::with_id(
            "taicho_new_profile",
            "New Profile",
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyN)),
        ),
        &MenuItem::with_id(
            "taicho_disconnect",
            "Disconnect",
            true,
            Some(Accelerator::new(
                Some(Modifiers::SUPER | Modifiers::SHIFT),
                Code::KeyD,
            )),
        ),
        &PredefinedMenuItem::close_window(None),
    ]);

    let view_menu = Submenu::new("View", true);
    let _ = view_menu.append_items(&[
        &MenuItem::with_id(
            "taicho_cmd_palette",
            "Command Palette\u{2026}",
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyK)),
        ),
        &MenuItem::with_id(
            "taicho_refresh",
            "Refresh",
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyR)),
        ),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id(
            "taicho_peers",
            "Peers",
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::Digit1)),
        ),
        &MenuItem::with_id(
            "taicho_sessions",
            "Sessions",
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::Digit2)),
        ),
        &MenuItem::with_id(
            "taicho_workspaces",
            "Workspaces",
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::Digit3)),
        ),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id(
            "taicho_focus_search",
            "Focus Search",
            true,
            Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyF)),
        ),
    ]);

    let _ = menu.append_items(&[&app_menu, &file_menu, &view_menu]);

    menu
}
