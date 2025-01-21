use crate::backend::component::mem_ctl::{KeyboardMmioCtl, VgaMmioCtl};
use crate::frontend::tab::Tab;
use crate::frontend::util::vga::get_pixels;
use crate::frontend::util::vga::{NUM_FONT_COLS, NUM_FONT_ROWS};
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use egui::{Context, Image, TextureHandle, Ui};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

pub struct Console {
    vga_buffer_receiver: Receiver<[u8; VgaMmioCtl::NUM_BYTES]>,
    vga_buffer: Option<[u8; VgaMmioCtl::NUM_BYTES]>,
    keyboard_buffer_sender: Sender<u8>,
    input_buffer: String,
    last_vga_update_instant: Instant,
    texture_handle: Option<TextureHandle>,
}

impl Console {
    pub fn new(
        vga_buffer_receiver: Receiver<[u8; VgaMmioCtl::NUM_BYTES]>,
        keyboard_buffer_sender: Sender<u8>,
    ) -> Self {
        Console {
            vga_buffer_receiver,
            vga_buffer: None,
            keyboard_buffer_sender,
            input_buffer: String::new(),
            last_vga_update_instant: Instant::now(),
            texture_handle: None,
        }
    }
}

impl Tab for Console {
    fn name(&self) -> &'static str {
        "🗖 Console"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        while let Ok(vga_buffer) = self.vga_buffer_receiver.try_recv() {
            self.vga_buffer = Some(vga_buffer);
        }

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
        if let Some(vga_buffer) = self.vga_buffer {
            if self.last_vga_update_instant.elapsed().as_millis() > 16 {
                self.texture_handle = Some(get_pixels(ctx, &vga_buffer));
                self.last_vga_update_instant = Instant::now();
            }
        }
        if let Some(texture_handle) = self.texture_handle.clone() {
            ui.add(Image::from_texture((
                texture_handle.id(),
                [
                    VgaMmioCtl::NUM_COLS as f32 * NUM_FONT_COLS as f32,
                    VgaMmioCtl::NUM_ROWS as f32 * NUM_FONT_ROWS as f32,
                ]
                .into(),
            )));
        }

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

        self.input_buffer
            .as_bytes()
            .iter()
            .for_each(|byte| self.keyboard_buffer_sender.try_send(*byte).unwrap());
        self.input_buffer.clear();
    }
}
