use eframe::egui;

pub struct Canvas {
    canvas_size: f32,
    active_rect: egui::Rect,
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
        if let Some(storage) = cc.storage {
            unimplemented!();
        }

        Default::default()
    }

    pub const ACTIVE_RECT_MARGIN: f32 = 50.0;

    pub const CANVAS_SIZE_DEFAULT: f32 = 50.0;

    pub fn init_active_rect(&self, pos: egui::Pos2) -> egui::Rect {
        egui::Rect::from_min_size(
            pos + egui::vec2(Self::ACTIVE_RECT_MARGIN, Self::ACTIVE_RECT_MARGIN),
            self.canvas_face(),
        )
    }

    pub fn canvas_face(&self) -> egui::Vec2 {
        self.preference.aspect_ratio * self.canvas_size
    }

    pub fn update_window_size(&mut self, frame: &mut eframe::Frame) {
        let canvas_face = self.canvas_face();

        frame.set_window_size(egui::vec2(
            canvas_face.x + Self::ACTIVE_RECT_MARGIN * 2.0,
            canvas_face.y + Self::ACTIVE_RECT_MARGIN * 2.0,
        ));
    }
}

impl eframe::App for Canvas {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let pos = egui::Pos2::default();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.menu_button("Preference", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Aspect Ratio: ");
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
                ui.label(format!("Canvas Size: {}", self.canvas_size));
                ui.horizontal(|ui| {
                    ui.label("Zoom ratio");
                    ui.add(egui::Slider::new(
                        &mut self.preference.zoom_ratio,
                        0.1..=5.0,
                    ));
                    if ui.button("+").clicked() {
                        self.canvas_size *= self.preference.zoom_ratio;
                        self.active_rect = self.init_active_rect(pos);
                        self.update_window_size(frame);
                    }
                    if ui.button("-").clicked() {
                        self.canvas_size /= self.preference.zoom_ratio;
                        self.active_rect = self.init_active_rect(pos);
                        self.update_window_size(frame);
                    }
                });
            });

            ui.scope(|ui| {
                let painter = ui.painter();
                painter.rect_stroke(
                    self.active_rect,
                    egui::Rounding::default(),
                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                );
            });
        });
    }
}
