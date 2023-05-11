mod osc;
mod ui;

use crate::ui::{Canvas, CanvasPreference};
use anyhow::Result;
use eframe::egui;

rust_i18n::i18n!("locales", fallback = "en-US");

fn main() -> Result<()> {
    egui_logger::init()?;

    let system_locale = if let Some(locale) = sys_locale::get_locale() {
        available_locales()
            .iter()
            .find(|l| l.to_string() == locale)
            .map(|l| l.to_string())
            .unwrap_or("en-US".to_string())
    } else {
        "en-US".to_string()
    };

    rust_i18n::set_locale(system_locale.as_str());

    let mut options = eframe::NativeOptions::default();

    let initial_canvas_face = Canvas::CANVAS_SIZE_DEFAULT * CanvasPreference::ASPECT_RATIO_DEFAULT;
    options.initial_window_size = Some(egui::vec2(
        initial_canvas_face.x + Canvas::ACTIVE_RECT_MARGIN * 2.0,
        initial_canvas_face.y + Canvas::ACTIVE_RECT_MARGIN * 2.0,
    ));

    Err(anyhow::anyhow!(
        "Couldn't start with {:?}",
        eframe::run_native(
            "VRC Canvas for stylus",
            options,
            Box::new(|cc| Box::new(Canvas::new(cc))),
        )
    ))
}
