use crate::frontend::core_gui_wrapper::RegisterData;
use crate::frontend::tab::Tab;
use crossbeam_channel::Receiver;
use egui::{Context, Ui};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

const NUM_ROWS: usize = 8;
const NUM_COLUMNS: usize = 4;

pub struct Register {
    data_receiver: Receiver<RegisterData>,
    data: Option<RegisterData>,
}

impl Tab for Register {
    fn name(&self) -> &'static str {
        "▣ Register"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .default_width(425.0)
            .default_height(250.0)
            .resizable(false)
            .show(ctx, |ui| {
                self.ui(ctx, ui);
            });
    }

    fn ui(&mut self, _ctx: &Context, ui: &mut Ui) {
        while let Ok(data) = self.data_receiver.try_recv() {
            self.data = Some(data);
        }
        if let Some(data) = &self.data {
            ui.vertical(|ui| {
                ui.strong("Machine Registers");
                ui.separator();
                egui::Grid::new("mreg_grid")
                    .num_columns(3)
                    .striped(true)
                    .show(ui, |ui| {
                        let mut size = ui.spacing().interact_size;
                        size.x = 90.0;

                        ui.strong("State");
                        ui.add_sized(size, |ui: &mut Ui| {
                            ui.text_edit_singleline(&mut data.state.clone())
                        });
                        ui.strong("IR");
                        ui.add_sized(size, |ui: &mut Ui| {
                            ui.text_edit_singleline(&mut data.ir.clone())
                        });
                        ui.strong("PC");
                        ui.add_sized(size, |ui: &mut Ui| {
                            ui.text_edit_singleline(&mut data.pc.clone())
                        });
                        ui.end_row();

                        ui.strong("MAR");
                        ui.text_edit_singleline(&mut data.mar.clone());
                        ui.strong("MrDR");
                        ui.text_edit_singleline(&mut data.mrdr.clone());
                        ui.strong("MwDR");
                        ui.text_edit_singleline(&mut data.mwdr.clone());
                        ui.end_row();
                    });

                ui.separator();

                ui.strong("RegFile");
                ui.separator();
                StripBuilder::new(ui)
                    .size(Size::remainder().at_least(200.0))
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                self.rf_table_ui(ui);
                            });
                        });
                    });
            });
        }
    }
}

impl Register {
    pub fn new(data_receiver: Receiver<RegisterData>) -> Self {
        Self {
            data_receiver,
            data: None,
        }
    }

    fn rf_table_ui(&self, ui: &mut Ui) {
        if let Some(data) = &self.data {
            let text_height = egui::TextStyle::Body
                .resolve(ui.style())
                .size
                .max(ui.spacing().interact_size.y);

            let mut table = TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .min_scrolled_height(0.0);

            for _ in 0..2 * NUM_COLUMNS {
                table = table.column(Column::auto());
            }

            table.body(|body| {
                body.rows(text_height, NUM_ROWS, |mut row| {
                    let row_index = row.index();

                    for col_index in 0..NUM_COLUMNS {
                        let reg_index = row_index * NUM_COLUMNS + col_index;
                        row.col(|ui| {
                            ui.strong(format!("x{}", reg_index));
                        });

                        row.col(|ui| {
                            ui.label(data.regfile[reg_index].clone());
                        });
                    }
                })
            });
        }
    }
}
