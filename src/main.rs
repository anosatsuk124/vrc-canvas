mod osc;
mod ui;

use crate::ui::{Canvas, CanvasPreference};
use anyhow::Result;
use eframe::egui;

fn main() -> Result<()> {
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
