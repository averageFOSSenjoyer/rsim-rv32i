use crate::backend::core::Core;
use crate::backend::util::types::Word;
use crate::frontend::tab::control::CoreCommand::{
    Reset, RunCycle, RunEnd, RunInstructions, RunUntilAddr,
};
use crate::frontend::tab::Tab;
use crossbeam_channel::{unbounded, Receiver, Sender};
use egui::{Context, Ui};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Control {
    breakpoints: Arc<Mutex<BTreeSet<Word>>>,
    command_sender: Sender<CoreCommand>,
    ack_receiver: Receiver<()>,
    ready: bool,
}

impl Control {
    pub fn new(core: Arc<Core>, breakpoints: Arc<Mutex<BTreeSet<Word>>>) -> Self {
        let command_channel = unbounded();
        let ack_channel = unbounded();
        let core_wrapper = Arc::new(CoreWrapper::new(core));
        thread::spawn(move || {
            core_wrapper.thread_loop(command_channel.1.clone(), ack_channel.0.clone())
        });

        Control {
            breakpoints,
            command_sender: command_channel.0,
            ack_receiver: ack_channel.1,
            ready: true,
        }
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
                if self.ready {
                    let mut core_command = None;
                    if ui.button("Next Cycle").clicked() {
                        core_command = Some(RunCycle)
                    }
                    if ui.button("Next Instruction").clicked() {
                        core_command = Some(RunInstructions)
                    }
                    if ui.button("Next Breakpoint").clicked() {
                        core_command = Some(RunUntilAddr(self.breakpoints.clone()))
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

enum CoreCommand {
    RunCycle,
    RunInstructions,
    RunUntilAddr(Arc<Mutex<BTreeSet<Word>>>),
    RunEnd,
    Reset,
}

struct CoreWrapper {
    core: Arc<Core>,
}

impl CoreWrapper {
    pub fn new(core: Arc<Core>) -> Self {
        Self { core }
    }

    pub fn thread_loop(&self, command_receiver: Receiver<CoreCommand>, ack_sender: Sender<()>) {
        loop {
            if let Ok(command) = command_receiver.recv() {
                match command {
                    RunCycle => {
                        self.core.run_cycle();
                    }
                    RunInstructions => {
                        self.core.run_instruction();
                    }
                    RunUntilAddr(addr) => {
                        self.core.run_until_addr(addr);
                    }
                    RunEnd => {
                        self.core.run_end();
                    }
                    Reset => {
                        self.core.reset();
                    }
                }
                ack_sender.try_send(()).unwrap();
            }
        }
    }
}
