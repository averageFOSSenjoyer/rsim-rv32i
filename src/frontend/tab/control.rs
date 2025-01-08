use crate::backend::core::Core;
use crate::backend::util::types::Word;
use crate::frontend::tab::Tab;
use egui::{Context, Ui};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

pub struct Control {
    core: Arc<Mutex<Core>>,
    breakpoints: Arc<Mutex<BTreeSet<Word>>>,
}

impl Control {
    pub fn new(core: Arc<Mutex<Core>>, breakpoints: Arc<Mutex<BTreeSet<Word>>>) -> Self {
        Control { core, breakpoints }
    }
}

impl Tab for Control {
    fn name(&self) -> &'static str {
        "Control"
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
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                // todo make all this non-blocking
                if ui.button("Next Cycle").clicked() {
                    self.core.lock().unwrap().run_cycle();
                }
                if ui.button("Next Instruction").clicked() {
                    self.core.lock().unwrap().run_instruction();
                }
                if ui.button("Next Breakpoint").clicked() {
                    self.core
                        .lock()
                        .unwrap()
                        .run_until_addr(self.breakpoints.clone());
                }
                if ui.button("Finish").clicked() {
                    self.core.lock().unwrap().run_end();
                }
                if ui.button("Reset").clicked() {
                    self.core.lock().unwrap().reset();
                }
            });
        });
    }
}
