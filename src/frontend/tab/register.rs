use crate::backend::core::Core;
use crate::frontend::tab::Tab;
use egui::{Context, Ui};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use std::sync::Arc;

const NUM_ROWS: usize = 8;
const NUM_COLUMNS: usize = 4;

pub struct Register {
    core: Arc<Core>,
}

impl Tab for Register {
    fn name(&self) -> &'static str {
        "Register"
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
                        ui.text_edit_singleline(&mut format!(
                            "{}",
                            self.core.control.lock().unwrap().state.clone()
                        ))
                    });
                    ui.strong("IR");
                    ui.add_sized(size, |ui: &mut Ui| {
                        ui.text_edit_singleline(&mut format!(
                            "{}",
                            self.core.ir.lock().unwrap().data_inner.clone()
                        ))
                    });
                    ui.strong("PC");
                    ui.add_sized(size, |ui: &mut Ui| {
                        ui.text_edit_singleline(&mut format!(
                            "{}",
                            self.core.pc.lock().unwrap().data_inner.clone()
                        ))
                    });
                    ui.end_row();

                    ui.strong("MAR");
                    ui.text_edit_singleline(&mut format!(
                        "{}",
                        self.core.mar.lock().unwrap().data_inner.clone()
                    ));
                    ui.strong("MrDR");
                    ui.text_edit_singleline(&mut format!(
                        "{}",
                        self.core.mrdr.lock().unwrap().data_inner.clone()
                    ));
                    ui.strong("MwDR");
                    ui.text_edit_singleline(&mut format!(
                        "{}",
                        self.core.mwdr.lock().unwrap().data_inner.clone()
                    ));
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

impl Register {
    pub fn new(core: Arc<Core>) -> Self {
        Self { core }
    }

    fn rf_table_ui(&mut self, ui: &mut Ui) {
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

                    let regfile = self.core.regfile.lock().unwrap();
                    let reg_value = regfile.registers.data[reg_index];
                    row.col(|ui| {
                        ui.label(reg_value.to_string());
                    });
                }
            })
        });
    }
}
