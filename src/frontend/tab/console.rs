use crate::backend::component::mem_ctl::{KeyboardMmioCtl, VgaMmioCtl};
use crate::frontend::tab::Tab;
use crate::frontend::util::vga::get_pixels;
use crate::frontend::util::vga::{NUM_FONT_COLS, NUM_FONT_ROWS};
use egui::{Context, Image, Ui};
use std::sync::{Arc, Mutex};

pub struct Console {
    vga_mmio_ctl: Arc<Mutex<VgaMmioCtl>>,
    keyboard_mmio_ctl: Arc<Mutex<KeyboardMmioCtl>>,
    input_buffer: String,
}

impl Console {
    pub fn new(
        vga_mmio_ctl: Arc<Mutex<VgaMmioCtl>>,
        keyboard_mmio_ctl: Arc<Mutex<KeyboardMmioCtl>>,
    ) -> Self {
        Console {
            vga_mmio_ctl,
            keyboard_mmio_ctl,
            input_buffer: String::new(),
        }
    }
}

impl Tab for Console {
    fn name(&self) -> &'static str {
        "Console"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .default_width(640.0)
            .default_height(480.0)
            .resizable(false)
            .show(ctx, |ui| {
                self.ui(ctx, ui);
            });
    }

    fn ui(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.strong("VGA Display").on_hover_ui(|ui| {
            ui.label(format!("VGA operates in text mode, blink bit is the not enabled.\nVGA Address ranges from 0x{:08X} to 0x{:08X}", VgaMmioCtl::BASE_ADDR, VgaMmioCtl::BASE_ADDR + VgaMmioCtl::NUM_BYTES as u32));
        });
        ui.separator();
        let texture_handle = get_pixels(ctx, self.vga_mmio_ctl.lock().unwrap().get_buffer());
        ui.add(Image::from_texture((
            texture_handle.id(),
            [
                VgaMmioCtl::NUM_COLS as f32 * NUM_FONT_COLS as f32,
                VgaMmioCtl::NUM_ROWS as f32 * NUM_FONT_ROWS as f32,
            ]
            .into(),
        )));

        ui.strong("Text Input").on_hover_ui(|ui| {
            ui.label(format!(
                "Status Address @ 0x{:08X}\nData Address @ 0x{:08X}",
                KeyboardMmioCtl::STATUS_ADDR,
                KeyboardMmioCtl::DATA_ADDR
            ));
        });
        ui.separator();
        ui.vertical_centered(|ui| {
            ui.text_edit_singleline(&mut self.input_buffer);
            ui.label(
                "! input will not be echoed, entering non-ascii character has undefined behavior",
            );
        });

        self.keyboard_mmio_ctl
            .lock()
            .unwrap()
            .append_to_buffer(self.input_buffer.as_bytes());
        self.input_buffer.clear();
    }
}
