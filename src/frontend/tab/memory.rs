use crate::backend::core::Core;
use crate::backend::util::byte::Bytes;
use crate::backend::util::types::{Byte, Word};
use crate::frontend::tab::Tab;
use egui::{Context, Ui};
use egui_extras::Size;
use egui_extras::TableBuilder;
use egui_extras::{Column, StripBuilder};
use egui_file::FileDialog;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const NUM_ROWS: usize = 0x200;

#[derive(PartialEq)]
enum AlignmentType {
    Byte,
    HalfWord,
    Word,
}

pub struct Memory {
    core: Arc<Core>,
    offset: usize,
    offset_str: String,
    alignment_type: AlignmentType,
    load_file_addr_str: String,
    opened_file: Option<PathBuf>,
    open_file_dialog: Option<FileDialog>,
    breakpoints: Arc<Mutex<BTreeSet<Word>>>,
}

impl Memory {
    pub fn new(core: Arc<Core>, breakpoints: Arc<Mutex<BTreeSet<Word>>>) -> Memory {
        Memory {
            core,
            offset: 0x40000000usize,
            offset_str: "0x40000000".to_string(),
            alignment_type: AlignmentType::Word,
            load_file_addr_str: "0x40000000".to_string(),
            opened_file: None,
            open_file_dialog: None,
            breakpoints,
        }
    }

    fn table_ui(&mut self, ui: &mut Ui) {
        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .animate_scrolling(true)
            .column(Column::auto().at_least(100.0))
            .column(Column::auto().at_least(100.0))
            .column(Column::auto().at_least(90.0));

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    egui::Sides::new().show(
                        ui,
                        |ui| {
                            ui.strong("Address");
                        },
                        |_| {},
                    );
                });
                header.col(|ui| {
                    ui.strong("Value");
                });
                header.col(|ui| {
                    ui.strong("Breakpoint");
                });
            })
            .body(|body| {
                body.rows(text_height, NUM_ROWS, |mut row| {
                    let byte_width = match self.alignment_type {
                        AlignmentType::Byte => 1u8,
                        AlignmentType::HalfWord => 2u8,
                        AlignmentType::Word => 4u8,
                    };

                    let row_index =
                        Word::from((row.index() * byte_width as usize + self.offset) as u32);
                    let mut value = Word::unknown();
                    let memctl = self.core.mem_ctl.lock().unwrap();
                    for i in 0..byte_width {
                        let addr = row_index + Byte::from(i);
                        if let Some(byte_value) = memctl.backend_mem.get(&addr) {
                            value[i as usize] = (*byte_value).into();
                        }
                    }

                    row.col(|ui| {
                        ui.label(format!("{}", row_index));
                    });

                    match self.alignment_type {
                        AlignmentType::Byte => {
                            row.col(|ui| {
                                ui.label(format!("{}", Byte::from(0xFFu8) & value));
                            });
                        }
                        AlignmentType::HalfWord => {
                            row.col(|ui| {
                                ui.label(format!("{}", Bytes::<2>::from(0xFFFFu16) & value));
                            });
                        }
                        AlignmentType::Word => {
                            row.col(|ui| {
                                ui.label(format!("{}", value));
                            });
                        }
                    };

                    row.col(|ui| {
                        let mut breakpoints = self.breakpoints.lock().unwrap();
                        let mut has_breakpoint = breakpoints.contains(&row_index);
                        ui.checkbox(&mut has_breakpoint, "");
                        if has_breakpoint {
                            if !breakpoints.contains(&row_index) {
                                breakpoints.insert(row_index);
                            }
                        } else {
                            breakpoints.remove(&row_index);
                        }
                    });
                });
            });
    }

    fn file_picker_ui(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Load address: ");
                ui.text_edit_singleline(&mut self.load_file_addr_str);
            });
            ui.horizontal(|ui| {
                ui.label("Load file: ");
                let fname = self
                    .opened_file
                    .clone()
                    .map(|f| f.into_os_string().into_string().unwrap())
                    .unwrap_or("None chosen".to_string());
                if fname.len() < 16 {
                    ui.label(fname);
                } else {
                    ui.label(format!("...{}", &fname[fname.len() - 16..]));
                }
                if ui.button("Choose file").clicked() {
                    let mut dialog = FileDialog::open_file(self.opened_file.clone());
                    dialog.open();
                    self.open_file_dialog = Some(dialog);
                }
                if ui.button("Load").clicked() {
                    let trimmed_load_file_addr_str =
                        self.load_file_addr_str.trim_start_matches("0x");
                    if let Ok(load_file_addr) =
                        usize::from_str_radix(trimmed_load_file_addr_str, 16)
                    {
                        if let Some(load_file) = self.opened_file.clone() {
                            if let Ok(data) = fs::read(load_file) {
                                self.core.load_bin(&data, Word::from(load_file_addr as u32));
                            }
                        }
                    } else {
                        self.load_file_addr_str =
                            format!("Failed to parse \"{}\"", self.load_file_addr_str).to_string();
                    }
                }
            });
        });

        if let Some(dialog) = &mut self.open_file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    self.opened_file = Some(file.to_path_buf());
                }
            }
        }
    }
}

impl Tab for Memory {
    fn name(&self) -> &'static str {
        "Memory"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .resizable([false, true])
            .default_width(300.0)
            .default_height(600.0)
            .show(ctx, |ui| {
                self.ui(ctx, ui);
            });
    }

    fn ui(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Memory Address: ");
                if ui.text_edit_singleline(&mut self.offset_str).lost_focus() {
                    let trimmed_offset_str = self.offset_str.trim_start_matches("0x");
                    self.offset =
                        usize::from_str_radix(trimmed_offset_str, 16).unwrap_or_else(|_| {
                            self.offset_str =
                                format!("Failed to parse \"{}\"", self.offset_str).to_string();
                            self.offset
                        });
                    if self.alignment_type == AlignmentType::Word {
                        self.offset -= self.offset % 4;
                    }
                    self.offset_str = format!("0x{:X}", self.offset);
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Alignment: ");
                ui.radio_value(&mut self.alignment_type, AlignmentType::Byte, "Byte");
                ui.radio_value(
                    &mut self.alignment_type,
                    AlignmentType::HalfWord,
                    "HalfWord",
                );
                ui.radio_value(&mut self.alignment_type, AlignmentType::Word, "Word");
            });

            ui.separator();

            StripBuilder::new(ui)
                .size(Size::remainder())
                .size(Size::exact(10.0))
                .size(Size::exact(50.0))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        self.table_ui(ui);
                    });
                    strip.cell(|ui| {
                        ui.separator();
                    });
                    strip.cell(|ui| {
                        self.file_picker_ui(ctx, ui);
                    });
                });
        });
    }
}
