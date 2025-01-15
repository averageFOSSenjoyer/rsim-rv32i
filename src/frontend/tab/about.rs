use crate::frontend::tab::Tab;
use egui::{Context, Ui};

pub struct About;

impl Tab for About {
    fn name(&self) -> &'static str {
        "✒ About"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .resizable(false)
            .show(ctx, |ui| {
                self.ui(ctx, ui);
            });
    }

    fn ui(&mut self, _ctx: &Context, ui: &mut Ui) {
        ui.label("Made by Frank Cai at the University of Illinois Urbana-Champaign for ECE 120");
        ui.separator();
        ui.label("Any feature request or contribution is welcomed by opening an issue on GitHub.");
        ui.separator();
        use egui::special_emojis::GITHUB;
        ui.horizontal(|ui| {
            ui.label("Source code available at ");
            ui.hyperlink_to(
                format!("{GITHUB} rsim-rv32i"),
                "https://github.com/averageFOSSenjoyer/rsim-rv32i",
            );
        });
        ui.horizontal(|ui| {
            ui.label("Simulation backend in ");
            ui.hyperlink_to(
                format!("{GITHUB} rsim-core"),
                "https://github.com/averageFOSSenjoyer/rsim-core",
            );
        });
        ui.horizontal(|ui| {
            ui.label("GUI frontend in ");
            ui.hyperlink_to(
                format!("{GITHUB} egui on GitHub"),
                "https://github.com/emilk/egui",
            );
        });
    }
}