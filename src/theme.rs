use eframe::egui;

pub const CONTENT_MAX_WIDTH: f32 = 1100.0;
pub const CONTENT_MIN_MARGIN: f32 = 32.0;

pub const FORM_MAX_WIDTH: f32 = 850.0;
pub const BUTTON_WIDTH: f32 = 320.0;

pub const SECTION_SPACING: f32 = 20.0;

pub const BACKGROUND: egui::Color32 = egui::Color32::from_rgb(244, 239, 228);

pub const CARD_BACKGROUND: egui::Color32 = egui::Color32::from_rgb(252, 250, 245);

pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(54, 52, 46);

pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(112, 106, 94);

pub const BORDER: egui::Color32 = egui::Color32::from_rgb(162, 181, 153);

pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(93, 122, 86);

pub const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(79, 107, 73);

pub const ACCENT_WARM: egui::Color32 = egui::Color32::from_rgb(181, 126, 72);

pub fn configure(ctx: &egui::Context) {
    // On ne dépend pas du thème sombre du système.
    ctx.set_theme(egui::Theme::Light);

    let mut visuals = egui::Visuals::light();

    // Fond principal
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = BACKGROUND;

    // Texte
    visuals.override_text_color = Some(TEXT_PRIMARY);

    // Widgets
    visuals.widgets.inactive.corner_radius = 8.0.into();
    visuals.widgets.hovered.corner_radius = 8.0.into();
    visuals.widgets.active.corner_radius = 8.0.into();

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style_of(egui::Theme::Light)).clone();

    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(16.0, 10.0);

    ctx.set_style_of(egui::Theme::Light, style);
}

pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(CARD_BACKGROUND)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(12.0)
        .inner_margin(20.0)
}
