use crate::frontend::core_gui_wrapper::ControlCommand;
use crate::frontend::core_gui_wrapper::ControlCommand::*;
use crate::frontend::tab::Tab;
use crossbeam_channel::{Receiver, Sender};
use egui::{Context, Ui};

pub struct Control {
    command_sender: Sender<ControlCommand>,
    ack_receiver: Receiver<()>,
    ready: bool,
}

impl Control {
    pub fn new(
        command_sender: Sender<ControlCommand>,
        ack_receiver: Receiver<()>,
    ) -> Self {
        Control {
            command_sender,
            ack_receiver,
            ready: true,
        }
    }
}

impl Tab for Control {
    fn name(&self) -> &'static str {
        "🖮 Control"
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
                if self.ready {
                    let mut core_command = None;
                    if ui.button("Next Cycle").clicked() {
                        core_command = Some(RunCycle)
                    }
                    if ui.button("Next Instruction").clicked() {
                        core_command = Some(RunInstructions)
                    }
                    if ui.button("Next Breakpoint").clicked() {
                        core_command = Some(RunUntilAddr)
                    }
                    if ui.button("Finish").clicked() {
                        core_command = Some(RunEnd)
                    }
                    if ui.button("Reset").clicked() {
                        core_command = Some(Reset)
                    }
                    if let Some(command) = core_command {
                        self.ready = false;
                        self.command_sender.try_send(command).unwrap()
                    }
                } else {
                    ui.spinner();
                }
            });
        });
        if self.ack_receiver.try_recv().is_ok() {
            self.ready = true;
        }
    }
}