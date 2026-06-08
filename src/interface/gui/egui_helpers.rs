#![allow(dead_code)]

#[cfg(feature = "gui")]
pub mod helpers {
    use eframe::egui;

    /// Returns standard native window options for a transparent/resizable desktop window.
    /// The window is centered on the primary display when possible (requires the
    /// "sys-info"/"platform-native" feature to be enabled alongside "gui" on the
    /// rcommon dependency for screen resolution query).
    pub fn get_default_options(width: f32, height: f32, transparent: bool) -> eframe::NativeOptions {
        let mut viewport = egui::ViewportBuilder::default()
            .with_transparent(transparent)
            .with_decorations(true)
            .with_inner_size(egui::vec2(width, height))
            .with_min_inner_size(egui::vec2(350.0, 250.0));

        // Best-effort center using rCommon's cross-platform screen resolution.
        #[cfg(feature = "sys-info")]
        {
            let (sw, sh) = crate::platform::native::sys_info::get_system_screen_resolution();
            let w = width as i32;
            let h = height as i32;
            let x = ((sw - w).max(0) / 2) as f32;
            let y = ((sh - h).max(0) / 2) as f32;
            viewport = viewport.with_position(egui::pos2(x, y));
        }

        eframe::NativeOptions {
            viewport,
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

    /// A premium, reusable container widget that draws a glassmorphic card
    /// with a solid accent-colored vertical stripe on the left margin.
    pub struct AccentCard<'a, R> {
        title: Option<&'a str>,
        accent_color: egui::Color32,
        add_contents: Box<dyn FnOnce(&mut egui::Ui) -> R + 'a>,
    }

    impl<'a, R> AccentCard<'a, R> {
        pub fn new(add_contents: impl FnOnce(&mut egui::Ui) -> R + 'a) -> Self {
            Self {
                title: None,
                accent_color: egui::Color32::from_rgb(0, 245, 255), // Sleek default cyan
                add_contents: Box::new(add_contents),
            }
        }

        pub fn with_title(mut self, title: &'a str) -> Self {
            self.title = Some(title);
            self
        }

        pub fn with_accent_color(mut self, color: egui::Color32) -> Self {
            self.accent_color = color;
            self
        }

        /// Show the card on the given UI.
        pub fn show(self, ui: &mut egui::Ui) -> egui::InnerResponse<R> {
            let accent_color = self.accent_color;
            let title = self.title;
            let add_contents = self.add_contents;

            // Frame for the glassmorphic card: dark translucent background, rounded corners.
            let card_frame = egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 20, 180))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)))
                .rounding(8.0)
                .inner_margin(egui::Margin {
                    left: 16.0,
                    right: 12.0,
                    top: 10.0,
                    bottom: 10.0,
                });

            card_frame.show(ui, |ui| {
                // Get the rect of the frame we just rendered (or are rendering)
                let rect = ui.max_rect();
                
                // Draw the left accent vertical stripe
                let stripe_width = 4.0;
                let stripe_rect = egui::Rect::from_min_max(
                    egui::pos2(rect.min.x + 4.0, rect.min.y + 8.0),
                    egui::pos2(rect.min.x + 4.0 + stripe_width, rect.max.y - 8.0),
                );
                
                // Paint the stripe
                ui.painter().rect_filled(
                    stripe_rect,
                    2.0, // rounding
                    accent_color,
                );

                // Now lay out the inner contents
                ui.vertical(|ui| {
                    if let Some(t) = title {
                        ui.horizontal(|ui| {
                            ui.strong(egui::RichText::new(t).color(accent_color));
                        });
                        ui.add_space(4.0);
                    }
                    add_contents(ui)
                }).inner
            })
        }
    }

    /// A premium button that draws an accent-colored border when hovered/focused,
    /// or filled background when clicked, matching the rApps ecosystem TUI styling.
    pub struct AccentButton<'a> {
        text: &'a str,
        accent_color: egui::Color32,
        focused: bool,
    }

    impl<'a> AccentButton<'a> {
        pub fn new(text: &'a str) -> Self {
            Self {
                text,
                accent_color: egui::Color32::from_rgb(0, 245, 255),
                focused: false,
            }
        }

        pub fn focused(mut self, focused: bool) -> Self {
            self.focused = focused;
            self
        }

        pub fn accent_color(mut self, color: egui::Color32) -> Self {
            self.accent_color = color;
            self
        }
    }

    impl<'a> egui::Widget for AccentButton<'a> {
        fn ui(self, ui: &mut egui::Ui) -> egui::Response {
            let text = egui::RichText::new(self.text).strong();
            
            // Layout button and configure interaction using standard egui button internally
            let button = egui::Button::new(text)
                .rounding(8.0)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)));
            
            ui.scope(|ui| {
                let active = self.focused;
                if active {
                    let visuals = &mut ui.style_mut().visuals;
                    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(self.accent_color.r(), self.accent_color.g(), self.accent_color.b(), 30);
                    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.5, self.accent_color);
                    visuals.widgets.inactive.fg_stroke.color = self.accent_color;
                }
                
                let visuals = &mut ui.style_mut().visuals;
                visuals.widgets.hovered.bg_fill = egui::Color32::from_rgba_unmultiplied(self.accent_color.r(), self.accent_color.g(), self.accent_color.b(), 50);
                visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, self.accent_color);
                visuals.widgets.hovered.fg_stroke.color = self.accent_color;
                
                visuals.widgets.active.bg_fill = self.accent_color;
                visuals.widgets.active.fg_stroke.color = egui::Color32::BLACK;
                
                ui.add(button)
            }).inner
        }
    }

    /// A premium horizontal tab bar selector widget for egui.
    pub struct AccentTabs<'a, T: PartialEq + Clone> {
        selected: &'a mut T,
        tabs: &'a [(T, &'a str)],
        accent_color: egui::Color32,
    }

    impl<'a, T: PartialEq + Clone> AccentTabs<'a, T> {
        pub fn new(selected: &'a mut T, tabs: &'a [(T, &'a str)]) -> Self {
            Self {
                selected,
                tabs,
                accent_color: egui::Color32::from_rgb(0, 245, 255),
            }
        }

        pub fn with_accent_color(mut self, color: egui::Color32) -> Self {
            self.accent_color = color;
            self
        }
    }

    impl<'a, T: PartialEq + Clone> egui::Widget for AccentTabs<'a, T> {
        fn ui(self, ui: &mut egui::Ui) -> egui::Response {
            ui.horizontal(|ui| {
                let mut final_response: Option<egui::Response> = None;

                for (value, label) in self.tabs {
                    let is_selected = *self.selected == *value;
                    
                    let res = ui.scope(|ui| {
                        if is_selected {
                            let visuals = &mut ui.style_mut().visuals;
                            visuals.widgets.inactive.fg_stroke.color = self.accent_color;
                            visuals.widgets.hovered.fg_stroke.color = self.accent_color;
                        }
                        
                        let text = egui::RichText::new(*label).strong();
                        ui.selectable_label(is_selected, text)
                    }).inner;

                    if res.clicked() {
                        *self.selected = value.clone();
                    }

                    if let Some(r) = final_response.as_mut() {
                        *r = r.union(res);
                    } else {
                        final_response = Some(res);
                    }
                }

                final_response.unwrap_or_else(|| ui.label(""))
            }).inner
        }
    }
}

#[cfg(all(feature = "gui", test))]
mod tests {
    use super::helpers::*;
    use eframe::egui;

    #[test]
    fn test_gui_helpers_compilation() {
        let ctx = egui::Context::default();
        apply_glassmorphism_style(&ctx);

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _card_res = AccentCard::new(|ui| {
                    ui.label("Card Content");
                })
                .with_title("Test Title")
                .show(ui);

                let _btn_res = ui.add(AccentButton::new("Click Me").focused(true));

                #[derive(PartialEq, Clone)]
                enum MockTab {
                    One,
                    Two,
                }
                let mut current_tab = MockTab::One;
                let _tabs_res = ui.add(AccentTabs::new(
                    &mut current_tab,
                    &[(MockTab::One, "One"), (MockTab::Two, "Two")],
                ));
            });
        });
    }
}

