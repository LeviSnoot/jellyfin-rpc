use dioxus::prelude::*;
use dioxus_desktop::{
    trayicon::{
        init_tray_icon,
        menu::{Menu, MenuItem, PredefinedMenuItem},
        Icon,
    },
    use_muda_event_handler, use_tray_menu_event_handler, use_window,
};
use std::rc::Rc;

use crate::agent::{AgentHandle, AgentStatus, Command};

/// Draws a simple round purple gradient icon, so no image assets are needed.
pub fn icon_rgba(size: u32) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let c = size as f32 / 2.0;

    for y in 0..size {
        for x in 0..size {
            let (dx, dy) = (x as f32 + 0.5 - c, y as f32 + 0.5 - c);
            let dist = (dx * dx + dy * dy).sqrt();
            let t = y as f32 / size as f32;
            let alpha = ((c - dist).clamp(0.0, 1.0) * 255.0) as u8;
            // Jellyfin-ish purple to blue
            rgba.extend_from_slice(&[
                (170.0 - 90.0 * t) as u8,
                (91.0 + 70.0 * t) as u8,
                (195.0 + 20.0 * t) as u8,
                alpha,
            ]);
        }
    }

    rgba
}

pub fn status_text(status: &AgentStatus) -> String {
    if status.config_error.is_some() {
        "Not configured".to_string()
    } else if status.paused {
        "Paused".to_string()
    } else if !status.discord_running {
        "Waiting for Discord".to_string()
    } else if !status.now_playing.is_empty() {
        status.now_playing.clone()
    } else if status.ipc_connected {
        "Connected to Discord, nothing playing".to_string()
    } else {
        "Connecting to Discord".to_string()
    }
}

#[derive(Clone)]
struct TrayItems {
    status: MenuItem,
    pause: MenuItem,
}

/// Creates the tray icon and wires its menu to the window and the agent.
pub fn use_tray(agent: AgentHandle, status: Signal<AgentStatus>) {
    let window = use_window();

    let items = use_hook(|| {
        let items = TrayItems {
            status: MenuItem::with_id("status", "Starting...", false, None),
            pause: MenuItem::with_id("pause", "Pause RPC", true, None),
        };
        let open = MenuItem::with_id("open", "Open Window", true, None);
        let quit = MenuItem::with_id("quit", "Quit", true, None);

        let menu = Menu::new();
        menu.append_items(&[
            &items.status,
            &PredefinedMenuItem::separator(),
            &open,
            &items.pause,
            &PredefinedMenuItem::separator(),
            &quit,
        ])
        .expect("failed to build tray menu");

        let icon = Icon::from_rgba(icon_rgba(32), 32, 32).ok();
        init_tray_icon(menu, icon);

        items
    });

    use_effect(move || {
        let status = status();
        let text = status_text(&status);
        items.status.set_text(format!("Jellyfin-RPC: {text}"));
        items
            .pause
            .set_text(if status.paused { "Resume RPC" } else { "Pause RPC" });
    });

    let on_menu = Rc::new(move |id: &str| match id {
        "open" => {
            window.set_visible(true);
            window.set_focus();
        }
        "pause" => {
            let paused = status.peek().paused;
            agent.send(Command::SetPaused(!paused));
        }
        "quit" => {
            agent.shutdown();
            std::process::exit(0);
        }
        _ => {}
    });

    // Both hooks are registered because dioxus-desktop and muda share a single global menu
    // event handler, and which of the two receives tray menu clicks depends on setup order.
    let handler = on_menu.clone();
    use_muda_event_handler(move |event| handler(event.id.0.as_str()));
    use_tray_menu_event_handler(move |event| on_menu(event.id.0.as_str()));
}
