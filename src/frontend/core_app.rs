use crate::backend::core::Core;
use crate::frontend::tab::console::Console;
use crate::frontend::tab::control::Control;
use crate::frontend::tab::datapath::Datapath;
use crate::frontend::tab::memory::Memory;
use crate::frontend::tab::register::Register;
use crate::frontend::tab::setting::Setting;
use crate::frontend::tab::Tab;
use egui::ScrollArea;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::Mutex;

pub struct CoreApp {
    widgets: Vec<Box<dyn Tab>>,
    opened_widget_by_name: BTreeSet<String>,
}

impl Default for CoreApp {
    fn default() -> Self {
        let core = Arc::new(Core::new(1, None));
        let breakpoints = Arc::new(Mutex::new(BTreeSet::new()));
        let vga_mmio_ctl = core.vga_mmio_ctl.clone();
        let keyboard_mmio_ctl = core.keyboard_mmio_ctl.clone();

        Self {
            widgets: vec![
                Box::new(Control::new(core.clone(), breakpoints.clone())),
                Box::new(Memory::new(core.clone(), breakpoints.clone())),
                Box::new(Register::new(core.clone())),
                Box::new(Datapath::new(core.clone())),
                Box::new(Console::new(vga_mmio_ctl, keyboard_mmio_ctl)),
                Box::new(Setting::new()),
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
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
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
