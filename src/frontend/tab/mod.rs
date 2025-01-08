use egui::Context;

pub mod console;
pub mod control;
pub mod datapath;
pub mod memory;
pub mod register;
pub mod setting;

pub trait Tab {
    fn name(&self) -> &'static str;
    fn show(&mut self, ctx: &Context, open: &mut bool);
    fn ui(&mut self, ctx: &Context, ui: &mut egui::Ui);
}
