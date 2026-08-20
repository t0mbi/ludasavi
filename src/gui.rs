mod app;
mod badge;
mod button;
mod common;
mod editor;
mod file_tree;
mod font;
mod game_list;
mod icon;
mod modal;
mod notification;
mod popup_menu;
mod screen;
mod search;
mod shortcuts;
mod style;
mod toast;
mod undoable;
mod widget;

use iced::Size;

use self::app::App;
pub use self::common::Flags;

fn app_icon() -> Option<iced::window::Icon> {
    let buffer = image::load_from_memory(include_bytes!("../assets/icon.png")).ok()?;
    let buffer = buffer.to_rgba8();
    let width = buffer.width();
    let height = buffer.height();
    let dynamic_image = image::DynamicImage::ImageRgba8(buffer);
    iced::window::icon::from_rgba(dynamic_image.into_bytes(), width, height).ok()
}

pub(crate) fn main_window_settings() -> iced::window::Settings {
    iced::window::Settings {
        min_size: Some(Size::new(800.0, 600.0)),
        exit_on_close_request: false,
        #[cfg(target_os = "linux")]
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: std::env::var(crate::prelude::ENV_LINUX_APP_ID)
                .unwrap_or_else(|_| crate::prelude::LINUX_APP_ID.to_string()),
            ..Default::default()
        },
        icon: app_icon(),
        ..Default::default()
    }
}

/// Small borderless popup used for backup/restore activity toasts.
/// The window is sized to roughly fit the longer of the toast's title/subtitle text.
pub(crate) fn toast_window_settings(width_hint: usize) -> iced::window::Settings {
    let width = (width_hint as f32 * 7.6 + 78.0).clamp(300.0, 380.0);

    iced::window::Settings {
        size: Size::new(width, 92.0),
        min_size: None,
        max_size: None,
        resizable: false,
        decorations: false,
        transparent: true,
        level: iced::window::Level::AlwaysOnTop,
        exit_on_close_request: false,
        position: iced::window::Position::SpecificWith(|window_size, screen_size| {
            let margin = 16.0;
            iced::Point::new(
                screen_size.width - window_size.width - margin,
                screen_size.height - window_size.height - margin,
            )
        }),
        #[cfg(target_os = "linux")]
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: std::env::var(crate::prelude::ENV_LINUX_APP_ID)
                .unwrap_or_else(|_| crate::prelude::LINUX_APP_ID.to_string()),
            ..Default::default()
        },
        icon: app_icon(),
        ..Default::default()
    }
}

pub fn run(flags: Flags) {
    #[cfg(windows)]
    crate::tray::init();

    let app = iced::daemon(move || App::new(flags.clone()), App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .title(App::title)
        .executor::<app::Executor>()
        .settings(iced::Settings {
            default_font: font::TEXT,
            ..Default::default()
        });

    if let Err(e) = app.run() {
        log::error!("Failed to initialize GUI: {e:?}");
        eprintln!("Failed to initialize GUI: {e:?}");

        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_description(e.to_string())
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    }
}
