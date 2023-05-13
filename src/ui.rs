use eframe::egui;
use rust_i18n::t;

use crate::osc::{self, pen_handle};

pub struct Canvas {
    canvas_size: f32,
    active_rect: egui::Rect,
    pen_handler: Option<pen_handle::PenHandler>,
    osc_started: bool,
    preference: CanvasPreference,
}

impl Default for Canvas {
    fn default() -> Self {
        let pos = egui::Pos2::default();

        let preference = CanvasPreference::default();

        Self {
            canvas_size: Self::CANVAS_SIZE_DEFAULT,
            active_rect: egui::Rect::from_min_size(
                pos + egui::vec2(Self::ACTIVE_RECT_MARGIN, Self::ACTIVE_RECT_MARGIN),
                egui::vec2(
                    preference.aspect_ratio.x * Self::CANVAS_SIZE_DEFAULT,
                    preference.aspect_ratio.y * Self::CANVAS_SIZE_DEFAULT,
                ),
            ),
            pen_handler: None,
            osc_started: false,
            preference,
        }
    }
}

pub struct CanvasPreference {
    aspect_ratio: egui::Vec2,
    zoom_ratio: f32,
}

impl Default for CanvasPreference {
    fn default() -> Self {
        Self {
            aspect_ratio: Self::ASPECT_RATIO_DEFAULT,
            zoom_ratio: Self::ZOOM_RATIO_DEFAULT,
        }
    }
}

impl CanvasPreference {
    pub const ZOOM_RATIO_DEFAULT: f32 = 2.0;
    pub const ASPECT_RATIO_DEFAULT: egui::Vec2 = egui::vec2(16.0, 9.0);
}

impl Canvas {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(_storage) = cc.storage {
            // TODO: load preference from storage
        }

        Default::default()
    }

    pub const ACTIVE_RECT_MARGIN: f32 = 50.0;

    pub const CANVAS_SIZE_DEFAULT: f32 = 50.0;

    const DEFAULT_POS: egui::Pos2 = egui::pos2(0f32, 0f32);

    fn init_active_rect(&self, pos: Option<egui::Pos2>) -> egui::Rect {
        match pos {
            Some(p) => egui::Rect::from_min_size(
                p + egui::vec2(Self::ACTIVE_RECT_MARGIN, Self::ACTIVE_RECT_MARGIN),
                self.canvas_face(),
            ),
            None => egui::Rect::from_min_size(
                Self::DEFAULT_POS + egui::vec2(Self::ACTIVE_RECT_MARGIN, Self::ACTIVE_RECT_MARGIN),
                self.canvas_face(),
            ),
        }
    }

    fn canvas_face(&self) -> egui::Vec2 {
        self.preference.aspect_ratio * self.canvas_size
    }

    fn update_window_size(&mut self, frame: &mut eframe::Frame) {
        let canvas_face = self.canvas_face();

        frame.set_window_size(egui::vec2(
            canvas_face.x + Self::ACTIVE_RECT_MARGIN * 2.0,
            canvas_face.y + Self::ACTIVE_RECT_MARGIN * 2.0,
        ));
    }

    pub fn from_absolute_to_relative(&self, pos: egui::Pos2) -> Option<egui::Pos2> {
        let relative = (pos - self.active_rect.min).to_pos2();

        if relative.x < 0.0 || relative.y < 0.0 {
            return None;
        }

        Some(relative)
    }

    fn process_with_position(&mut self, pos: egui::Pos2) {
        let handler = match pen_handle::PEN_HANDLER.get() {
            Some(handler) => handler,
            None => return,
        };

        let target_state = pen_handle::PenState::drawing_from_pos(pos);

        let new_handler = handler.new_handler(target_state);
        pen_handle::PEN_HANDLER.set(new_handler);

        tokio::spawn(async move { handler.eval() });
    }
}

fn get_interact_pos(input_state: &egui::InputState) -> Option<egui::Pos2> {
    let pointer = &input_state.pointer;
    let is_down = pointer.any_down();
    let is_moving = pointer.is_moving();

    if !is_down || !is_moving {
        return None;
    }

    pointer.interact_pos()
}

impl eframe::App for Canvas {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.menu_button(t!("Preference.Preference"), |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}: ", t!("Preference.AspectRatio")));
                    let mut width = self.preference.aspect_ratio.x.to_string();
                    let mut height = self.preference.aspect_ratio.y.to_string();
                    ui.text_edit_singleline(&mut width);
                    ui.text_edit_singleline(&mut height);
                    self.preference.aspect_ratio = (
                        width.parse().unwrap_or(self.preference.aspect_ratio.x),
                        height.parse().unwrap_or(self.preference.aspect_ratio.y),
                    )
                        .into();
                });
                ui.label(format!(
                    "{}: {}",
                    t!("Preference.CanvasSize"),
                    self.canvas_size
                ));
                ui.horizontal(|ui| {
                    ui.label(format!("{}: ", t!("Preference.ZoomRatio")));
                    ui.add(egui::Slider::new(
                        &mut self.preference.zoom_ratio,
                        0.1..=5.0,
                    ));
                    if ui.button("+").clicked() {
                        self.canvas_size *= self.preference.zoom_ratio;
                        self.active_rect = self.init_active_rect(None);
                        self.update_window_size(frame);
                    }
                    if ui.button("-").clicked() {
                        self.canvas_size /= self.preference.zoom_ratio;
                        self.active_rect = self.init_active_rect(None);
                        self.update_window_size(frame);
                    }
                });
            });

            ui.menu_button(t!("Logs"), |ui| {
                egui_logger::logger_ui(ui);
            });

            if ui.button(t!("Start")).clicked() {
                self.osc_started = true;
            }

            ui.scope(|ui| {
                let painter = ui.painter();
                painter.rect_stroke(
                    self.active_rect,
                    egui::Rounding::default(),
                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                );

                if let Some(interact_pos) = ctx.input(get_interact_pos) {
                    painter.circle_stroke(
                        interact_pos,
                        5.0,
                        egui::Stroke::new(1.0, egui::Color32::WHITE),
                    );

                    let relative_pos = self.from_absolute_to_relative(interact_pos);
                    if let Some(relative_pos) = relative_pos {
                        if self.osc_started {
                            let state = pen_handle::PenState::drawing_from_pos(relative_pos);
                            if let Err(e) = osc::start_osc(state) {
                                log::error!("Failed to start osc: {}", e);
                                self.osc_started = false;
                            }
                        }
                        log::info!("Position in active rect: {:?}", relative_pos);
                        self.process_with_position(relative_pos);
                    }
                };
            });
        });
    }
}
