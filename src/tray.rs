//! Windows system tray icon: a right-click menu (Backup / Restore / Settings / Exit),
//! left-click to restore the main window, and a tooltip showing live progress.
//!
//! The tray icon must live on a dedicated thread that keeps pumping its own Win32
//! message queue - otherwise clicks and menu selections never get delivered.

use std::sync::{Mutex, OnceLock, mpsc};

use iced::{
    Subscription,
    futures::{Stream, stream},
};
use tray_icon::{
    Icon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    ShowWindow,
    Backup,
    Restore,
    Settings,
    Exit,
}

struct TraySetup {
    tooltip_tx: mpsc::Sender<String>,
    commands: Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<TrayCommand>>>,
}

static SETUP: OnceLock<TraySetup> = OnceLock::new();

/// Spawns the tray icon on its own dedicated OS thread. Call once at startup.
pub fn init() {
    let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
    let (tooltip_tx, tooltip_rx) = mpsc::channel::<String>();

    std::thread::spawn(move || run(command_tx, tooltip_rx));

    let _ = SETUP.set(TraySetup {
        tooltip_tx,
        commands: Mutex::new(Some(command_rx)),
    });
}

pub fn set_tooltip(text: impl Into<String>) {
    if let Some(setup) = SETUP.get() {
        let _ = setup.tooltip_tx.send(text.into());
    }
}

fn take_commands() -> Option<tokio::sync::mpsc::UnboundedReceiver<TrayCommand>> {
    SETUP.get()?.commands.lock().unwrap().take()
}

/// Subscription that emits a [`TrayCommand`] whenever the user interacts with the tray icon.
pub fn subscription() -> Subscription<TrayCommand> {
    Subscription::run(commands_stream)
}

fn commands_stream() -> impl Stream<Item = TrayCommand> {
    stream::unfold(take_commands(), |receiver| async move {
        let mut receiver = receiver?;
        let command = receiver.recv().await?;
        Some((command, Some(receiver)))
    })
}

fn load_icon() -> Option<Icon> {
    let buffer = image::load_from_memory(include_bytes!("../assets/icon.png")).ok()?;
    let buffer = buffer.to_rgba8();
    let (width, height) = (buffer.width(), buffer.height());
    Icon::from_rgba(buffer.into_raw(), width, height).ok()
}

const ID_BACKUP: &str = "backup";
const ID_RESTORE: &str = "restore";
const ID_SETTINGS: &str = "settings";
const ID_EXIT: &str = "exit";

fn run(command_tx: tokio::sync::mpsc::UnboundedSender<TrayCommand>, tooltip_rx: mpsc::Receiver<String>) {
    let menu = Menu::new();
    let _ = menu.append_items(&[
        &MenuItem::with_id(ID_BACKUP, "Backup Saves", true, None),
        &MenuItem::with_id(ID_RESTORE, "Restore Saves", true, None),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id(ID_SETTINGS, "Settings", true, None),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id(ID_EXIT, "Exit", true, None),
    ]);

    let mut builder = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Ludusavi")
        .with_menu_on_left_click(false);
    if let Some(icon) = load_icon() {
        builder = builder.with_icon(icon);
    }

    let tray_icon = match builder.build() {
        Ok(tray_icon) => tray_icon,
        Err(e) => {
            log::error!("Failed to create tray icon: {e:?}");
            return;
        }
    };

    let menu_events = MenuEvent::receiver();
    let tray_events = TrayIconEvent::receiver();

    loop {
        pump_windows_messages();

        while let Ok(event) = menu_events.try_recv() {
            let command = match event.id().as_ref() {
                ID_BACKUP => Some(TrayCommand::Backup),
                ID_RESTORE => Some(TrayCommand::Restore),
                ID_SETTINGS => Some(TrayCommand::Settings),
                ID_EXIT => Some(TrayCommand::Exit),
                _ => None,
            };
            if let Some(command) = command {
                let exiting = command == TrayCommand::Exit;
                let _ = command_tx.send(command);
                if exiting {
                    return;
                }
            }
        }

        while let Ok(event) = tray_events.try_recv() {
            if let TrayIconEvent::Click {
                button: tray_icon::MouseButton::Left,
                ..
            } = event
            {
                let _ = command_tx.send(TrayCommand::ShowWindow);
            }
        }

        while let Ok(tooltip) = tooltip_rx.try_recv() {
            let _ = tray_icon.set_tooltip(Some(tooltip));
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn pump_windows_messages() {
    use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage};

    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
