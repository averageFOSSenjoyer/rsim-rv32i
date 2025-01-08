use crate::frontend::tab::Tab;
use egui::Context;
use egui::Ui;

pub struct Setting {
    scaling: f32,
}

impl Setting {
    pub fn new() -> Self {
        Setting { scaling: 1.25 }
    }
}

impl Tab for Setting {
    fn name(&self) -> &'static str {
        "Setting"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .default_height(300.0)
            .default_width(400.0)
            .resizable(false)
            .show(ctx, |ui| {
                self.ui(ctx, ui);
            });
    }

    fn ui(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Scaling: ");
            ui.add(egui::Slider::new(&mut self.scaling, 1.0..=2.0));
        });

        if ui.button("Save").clicked() {
            ctx.set_pixels_per_point(self.scaling);
        }
    }
}
