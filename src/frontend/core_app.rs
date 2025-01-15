use crate::frontend::tab::about::About;
use crate::frontend::tab::register::Register;
use crate::backend::core::Core;
use crate::frontend::tab::console::Console;
use crate::frontend::tab::setting::Setting;
use crate::frontend::tab::Tab;
use egui::ScrollArea;
use std::collections::BTreeSet;
use std::sync::Arc;
use crossbeam_channel::unbounded;
use crate::frontend::core_gui_wrapper::CoreGuiWrapper;
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[cfg(target_arch = "wasm32")]
use wasm_thread as thread;
use crate::frontend::tab::control::Control;
use crate::frontend::tab::datapath::Datapath;
use crate::frontend::tab::memory::Memory;

pub struct CoreApp {
    widgets: Vec<Box<dyn Tab>>,
    opened_widget_by_name: BTreeSet<String>,
}

impl Default for CoreApp {
    fn default() -> Self {
        let core = Arc::new(Core::new(1, None));
        let console_vga_buffer_channel = unbounded();
        let console_keyboard_buffer_channel = unbounded();
        let control_command_channel = unbounded();
        let control_ack_channel = unbounded();
        let register_data_channel = unbounded();
        let breakpoint_channel = unbounded();
        let memory_channel = unbounded();
        let load_bin_channel = unbounded();
        let datapath_component_channel = unbounded();
        let mut core_wrapper = CoreGuiWrapper::new(
            core.clone(),
            console_vga_buffer_channel.0.clone(),
            console_keyboard_buffer_channel.1.clone(),
            control_command_channel.1.clone(),
            control_ack_channel.0.clone(),
            register_data_channel.0.clone(),
            breakpoint_channel.1.clone(),
            memory_channel.0.clone(),
            load_bin_channel.1.clone(),
            datapath_component_channel.0.clone(),
        );

        core_wrapper.send_update();
        // doesn't need to be joined
        thread::spawn(move || core_wrapper.event_loop());

        Self {
            widgets: vec![
                Box::new(Control::new(control_command_channel.0.clone(), control_ack_channel.1.clone())),
                Box::new(Memory::new(breakpoint_channel.0.clone(), memory_channel.1.clone(), load_bin_channel.0.clone())),
                Box::new(Register::new(register_data_channel.1.clone())),
                Box::new(Datapath::new(datapath_component_channel.1.clone())),
                Box::new(Console::new(console_vga_buffer_channel.1.clone(), console_keyboard_buffer_channel.0.clone())),
                Box::new(Setting::default()),
                Box::new(About {})
            ],
            opened_widget_by_name: Default::default(),
        }
    }
}

impl CoreApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn show_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("side_panel")
            .resizable(false)
            .default_width(125.0)
            .show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    let opened_widget_by_name = &mut self.opened_widget_by_name;
                    for widget in self.widgets.iter_mut() {
                        let mut is_open = opened_widget_by_name.contains(widget.name());
                        ui.toggle_value(&mut is_open, widget.name());
                        Self::set_open(opened_widget_by_name, widget.name(), is_open);
                    }
                });
            });
        });
    }

    fn show_widgets(&mut self, ctx: &egui::Context) {
        let opened_widget_by_name = &mut self.opened_widget_by_name;
        for widget in self.widgets.iter_mut() {
            let mut is_open = opened_widget_by_name.contains(widget.name());
            widget.show(ctx, &mut is_open);
            Self::set_open(opened_widget_by_name, widget.name(), is_open);
        }
    }

    fn set_open(opened_widget_by_name: &mut BTreeSet<String>, name: &'static str, is_open: bool) {
        if is_open {
            if !opened_widget_by_name.contains(name) {
                opened_widget_by_name.insert(name.to_string());
            }
        } else {
            opened_widget_by_name.remove(&name.to_string());
        }
    }
}

impl eframe::App for CoreApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
        style.interaction.tooltip_delay = 0.0;
        ctx.set_style(style);
        self.show_side_panel(ctx);
        self.show_widgets(ctx);
    }
}
