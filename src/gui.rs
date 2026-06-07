#![allow(dead_code)]

#[cfg(feature = "gui")]
pub mod helpers {
    use eframe::egui;

    /// Returns standard native window options for a transparent/resizable desktop window.
    pub fn get_default_options(width: f32, height: f32, transparent: bool) -> eframe::NativeOptions {
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_transparent(transparent)
                .with_decorations(true)
                .with_inner_size(egui::vec2(width, height))
                .with_min_inner_size(egui::vec2(350.0, 250.0)),
            ..Default::default()
        }
    }

    /// Configures global styling options to achieve a modern, premium dark UI.
    pub fn apply_glassmorphism_style(ctx: &egui::Context) {
        ctx.set_visuals(egui::Visuals::dark());
        let mut style = (*ctx.style()).clone();
        
        // Premium rounded edges
        style.visuals.window_rounding = 12.0.into();
        style.visuals.widgets.noninteractive.rounding = 8.0.into();
        style.visuals.widgets.inactive.rounding = 8.0.into();
        style.visuals.widgets.hovered.rounding = 8.0.into();
        style.visuals.widgets.active.rounding = 8.0.into();
        
        // Clean font settings
        ctx.set_style(style);
    }
}
