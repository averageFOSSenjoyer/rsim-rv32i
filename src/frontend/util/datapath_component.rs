use crate::backend::component::alu::{Alu, AluMux1, AluMux2};
use crate::backend::component::cmp::{Cmp, CmpMux};
use crate::backend::component::ir::IR;
use crate::backend::component::mem_addr_mux::MemAddrMux;
use crate::backend::component::mem_ctl::MemCtl;
use crate::backend::component::pc::{Pc, PcMux};
use crate::backend::component::regfile::{RegFile, RegFileMux};
use crate::backend::core::ComponentType;
use crate::frontend::util::datapath_net::DatapathNet::*;
use crate::frontend::util::datapath_net::{DatapathNet, DatapathNetDispalyer};
use eframe::emath::Align;
use egui::epaint::{PathShape, PathStroke};
use egui::{Color32, Frame, Painter, Response, Stroke, Ui, Widget};
use egui::{Layout, Pos2, Vec2};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct PortValue {
    pub name: String,
    pub value: String,
    pub associated_nets: HashSet<DatapathNet>,
}

impl PortValue {
    pub fn new(name: String, value: String, associated_nets: HashSet<DatapathNet>) -> Self {
        Self {
            name,
            value,
            associated_nets,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PortValues {
    pub inputs: Vec<PortValue>,
    pub outputs: Vec<PortValue>,
}

impl PortValues {
    pub fn new(inputs: Vec<PortValue>, outputs: Vec<PortValue>) -> Self {
        Self { inputs, outputs }
    }
}

#[derive(Clone)]
pub struct DatapathComponent {
    pub name: String,
    pub values: PortValues,
}

pub struct DatapathComponentWidget {
    pub datapath_component: DatapathComponent,
    pub window_pos2: [f32; 2],
    pub painter: Painter,
}

impl Widget for DatapathComponentWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        let name = self.datapath_component.name;
        let port_values = self.datapath_component.values;
        let on_hover = |ui: &mut Ui, port_value: &PortValue| {

            ui.label(port_value.name.clone()).on_hover_ui(|ui| {
                if port_value.value.contains('\n') {
                    ui.add_sized(Vec2::new(135.0, 10.0), |ui: &mut Ui| {
                        ui.text_edit_multiline(&mut port_value.value.clone())
                    });
                } else {
                    ui.add_sized(Vec2::new(90.0, 10.0), |ui: &mut Ui| {
                        ui.text_edit_singleline(&mut port_value.value.clone())
                    });
                }
                for associated_net in port_value.associated_nets.iter() {
                    let mut points = associated_net.get_points();
                    for point in points.iter_mut() {
                        *point += self.window_pos2.into();
                    }
                    self.painter.add(PathShape {
                        points,
                        closed: false,
                        fill: Color32::TRANSPARENT,
                        stroke: PathStroke::new(2.0, Color32::LIGHT_RED),
                    });
                }
            })
        };

        Frame::none()
            .inner_margin(5.0)
            .rounding(6.0)
            .stroke(Stroke::new(1.0, Color32::GRAY))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.strong(name.clone());
                    ui.separator();
                });
                ui.vertical_centered(|ui| {
                    egui::Grid::new(format!("{}_grid", name.clone()))
                        .num_columns(2)
                        .show(ui, |ui| {
                            for i in 0..port_values.inputs.len().max(port_values.outputs.len()) {
                                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                    port_values
                                        .inputs
                                        .get(i)
                                        .map(|port_value| on_hover(ui, port_value));
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    port_values
                                        .outputs
                                        .get(i)
                                        .map(|port_value| on_hover(ui, port_value));
                                });
                                ui.end_row();
                            }
                        });
                })
            })
            .response
    }
}

pub const GLOBAL_FRAME_WIDTH: f32 = 110.0;

impl ComponentType {
    pub(crate) fn get_frame_offset(&self) -> Pos2 {
        match *self {
            ComponentType::Alu => Pos2::new(910.0, 120.0),
            ComponentType::AluMux1 => Pos2::new(780.0, 310.0),
            ComponentType::AluMux2 => Pos2::new(780.0, 120.0),
            ComponentType::Cmp => Pos2::new(910.0, 430.0),
            ComponentType::CmpMux => Pos2::new(780.0, 480.0),
            ComponentType::Ir => Pos2::new(180.0, 120.0),
            ComponentType::MemAddrMux => Pos2::new(50.0, 430.0),
            ComponentType::MemCtl => Pos2::new(50.0, 120.0),
            ComponentType::Pc => Pos2::new(180.0, 550.0),
            ComponentType::PcMux => Pos2::new(50.0, 550.0),
            ComponentType::RegFile => Pos2::new(570.0, 120.0),
            ComponentType::RegFileMux => Pos2::new(410.0, 120.0),
        }
    }
    pub(crate) fn get_frame_size(&self) -> Vec2 {
        match *self {
            ComponentType::Alu => Vec2::new(GLOBAL_FRAME_WIDTH, 25.0),
            ComponentType::AluMux1 => Vec2::new(GLOBAL_FRAME_WIDTH, 25.0),
            ComponentType::AluMux2 => Vec2::new(GLOBAL_FRAME_WIDTH, 25.0),
            ComponentType::Cmp => Vec2::new(GLOBAL_FRAME_WIDTH, 25.0),
            ComponentType::CmpMux => Vec2::new(GLOBAL_FRAME_WIDTH, 25.0),
            ComponentType::Ir => Vec2::new(GLOBAL_FRAME_WIDTH, 75.0),
            ComponentType::MemAddrMux => Vec2::new(GLOBAL_FRAME_WIDTH, 25.0),
            ComponentType::MemCtl => Vec2::new(GLOBAL_FRAME_WIDTH, 75.0),
            ComponentType::Pc => Vec2::new(GLOBAL_FRAME_WIDTH, 25.0),
            ComponentType::PcMux => Vec2::new(GLOBAL_FRAME_WIDTH, 25.0),
            ComponentType::RegFile => Vec2::new(GLOBAL_FRAME_WIDTH, 25.0),
            ComponentType::RegFileMux => Vec2::new(GLOBAL_FRAME_WIDTH, 25.0),
        }
    }
}

pub trait DatapathComponentDisplayer {
    fn get_datapath_component(&self) -> DatapathComponent;
}

impl DatapathComponentDisplayer for Alu {
    fn get_datapath_component(&self) -> DatapathComponent {
        DatapathComponent {
            name: "ALU".to_string(),
            values: PortValues::new(
                vec![
                    PortValue::new(
                        "b".to_string(),
                        format!("0x{:X}", self.b.get_value()),
                        [AluMux2_out_Alu_b].into(),
                    ),
                    PortValue::new(
                        "a".to_string(),
                        format!("0x{:X}", self.a.get_value()),
                        [AluMux1_out_Alu_a].into(),
                    ),
                    PortValue::new(
                        "op".to_string(),
                        format!("0b{:3b}", self.op.get_value()),
                        [].into(),
                    ),
                ],
                vec![PortValue::new(
                    "out".to_string(),
                    format!("0x{:X}", self.out.get_value()),
                    [
                        Alu_out_MemAddrMux_alu_out,
                        Alu_out_PcMux_alu_out,
                        Alu_out_RegFileMux_Alu_out,
                    ]
                    .into(),
                )],
            ),
        }
    }
}

impl DatapathComponentDisplayer for AluMux1 {
    fn get_datapath_component(&self) -> DatapathComponent {
        DatapathComponent {
            name: "ALU Mux 1".to_string(),
            values: PortValues::new(
                vec![
                    PortValue::new(
                        "rs1_data".to_string(),
                        format!("0x{:X}", self.rs1.get_value()),
                        [RegFile_rs1_data_AluMux1_rs1_data].into(),
                    ),
                    PortValue::new(
                        "pc".to_string(),
                        format!("0x{:X}", self.pc.get_value()),
                        [Pc_out_AluMux1_pc].into(),
                    ),
                    PortValue::new(
                        "sel".to_string(),
                        format!("0b{:2b}", self.sel.get_value()),
                        [].into(),
                    ),
                ],
                vec![PortValue::new(
                    "out".to_string(),
                    format!("0x{:X}", self.out.get_value()),
                    [AluMux1_out_Alu_a].into(),
                )],
            ),
        }
    }
}

impl DatapathComponentDisplayer for AluMux2 {
    fn get_datapath_component(&self) -> DatapathComponent {
        DatapathComponent {
            name: "ALU Mux 2".to_string(),
            values: PortValues::new(
                vec![
                    PortValue::new(
                        "imm".to_string(),
                        format!("\
                        i_imm: 0x{:X}\n\
                        u_imm: 0x{:X}\n\
                        b_imm: 0x{:X}\n\
                        s_imm: 0x{:X}\n\
                        j_imm: 0x{:X}\
                        ",
                                self.i_imm.get_value(),
                                self.u_imm.get_value(),
                                self.b_imm.get_value(),
                                self.s_imm.get_value(),
                                self.j_imm.get_value(),
                        ),
                        [Ir_imm_AluMux2_imm].into(),
                    ),
                    PortValue::new(
                        "rs2_data".to_string(),
                        format!("0x{:X}", self.rs2.get_value()),
                        [RegFile_rs2_data_AluMux2_rs2_data].into(),
                    ),
                    PortValue::new(
                        "sel".to_string(),
                        format!("0b{:3b}", self.sel.get_value()),
                        [].into(),
                    ),
                ],
                vec![PortValue::new(
                    "out".to_string(),
                    format!("0x{:X}", self.out.get_value()),
                    [AluMux2_out_Alu_b].into(),
                )],
            ),
        }
    }
}

impl DatapathComponentDisplayer for CmpMux {
    fn get_datapath_component(&self) -> DatapathComponent {
        DatapathComponent {
            name: "CMP Mux".to_string(),
            values: PortValues::new(
                vec![
                    PortValue::new(
                        "rs2_data".to_string(),
                        format!("0x{:X}", self.rs2.get_value()),
                        [RegFile_rs2_data_CmpMux_rs2_data].into(),
                    ),
                    PortValue::new(
                        "i_imm".to_string(),
                        format!("0x{:X}", self.i_imm.get_value()),
                        [Ir_imm_CmpMux_i_imm].into(),
                    ),
                    PortValue::new(
                        "sel".to_string(),
                        format!("0x{:2b}", self.sel.get_value()),
                        [].into(),
                    ),
                ],
                vec![PortValue::new(
                    "out".to_string(),
                    format!("0x{:X}", self.out.get_value()),
                    [CmpMux_out_Cmp_b].into(),
                )],
            ),
        }
    }
}

impl DatapathComponentDisplayer for Cmp {
    fn get_datapath_component(&self) -> DatapathComponent {
        DatapathComponent {
            name: "CMP".to_string(),
            values: PortValues::new(
                vec![
                    PortValue::new(
                        "a".to_string(),
                        format!("0x{:X}", self.a.get_value()),
                        [RegFile_rs1_data_Cmp_a].into(),
                    ),
                    PortValue::new(
                        "b".to_string(),
                        format!("0x{:X}", self.b.get_value()),
                        [CmpMux_out_Cmp_b].into(),
                    ),
                    PortValue::new(
                        "op".to_string(),
                        format!("0x{:3b}", self.op.get_value()),
                        [].into(),
                    ),
                ],
                vec![PortValue::new(
                    "out".to_string(),
                    format!("0x{:X}", self.out.get_value()),
                    [Cmp_out_RegFileMux_Cmp_out].into(),
                )],
            ),
        }
    }
}

impl DatapathComponentDisplayer for IR {
    fn get_datapath_component(&self) -> DatapathComponent {
        DatapathComponent {
            name: "IR".to_string(),
            values: PortValues::new(
                vec![
                    PortValue::new(
                        "data".to_string(),
                        format!("0x{:X}", self.data.get_value()),
                        [MemCtl_rdata_Ir_data].into(),
                    ),
                    PortValue::new(
                        "load".to_string(),
                        format!("0b{:1b}", self.load.get_value()),
                        [].into(),
                    ),
                ],
                vec![
                    PortValue::new(
                        "imm".to_string(),
                        format!("\
                        i_imm: 0x{:X}\n\
                        u_imm: 0x{:X}\n\
                        b_imm: 0x{:X}\n\
                        s_imm: 0x{:X}\n\
                        j_imm: 0x{:X}\
                        ",
                                self.i_imm.get_value(),
                                self.u_imm.get_value(),
                                self.b_imm.get_value(),
                                self.s_imm.get_value(),
                                self.j_imm.get_value(),
                        ),
                        [Ir_imm_CmpMux_i_imm, Ir_imm_AluMux2_imm].into(),
                    ),
                    PortValue::new(
                        "rs1_idx".to_string(),
                        format!("0x{:X}", self.rs1.get_value()),
                        [Ir_rs1_idx_RegFile_rs1_idx].into(),
                    ),
                    PortValue::new(
                        "rs2_idx".to_string(),
                        format!("0x{:X}", self.rs2.get_value()),
                        [Ir_rs2_idx_RegFile_rs2_idx].into(),
                    ),
                    PortValue::new(
                        "rd_idx".to_string(),
                        format!("0x{:X}", self.rd.get_value()),
                        [Ir_rd_idx_RegFile_rd_idx].into(),
                    ),
                    PortValue::new(
                        "funct3".to_string(),
                        format!("0x{:X}", self.funct3.get_value()),
                        [].into(),
                    ),
                    PortValue::new(
                        "funct7".to_string(),
                        format!("0x{:X}", self.funct7.get_value()),
                        [].into(),
                    ),
                    PortValue::new(
                        "opcode".to_string(),
                        format!("0x{:X}", self.opcode.get_value()),
                        [].into(),
                    ),
                ],
            ),
        }
    }
}

impl DatapathComponentDisplayer for MemAddrMux {
    fn get_datapath_component(&self) -> DatapathComponent {
        DatapathComponent {
            name: "Mem Addr Mux".to_string(),
            values: PortValues::new(
                vec![
                    PortValue::new(
                        "pc".to_string(),
                        format!("0x{:X}", self.pc.get_value()),
                        [Pc_out_MemAddrMux_pc].into(),
                    ),
                    PortValue::new(
                        "alu_out".to_string(),
                        format!("0x{:X}", self.alu_out.get_value()),
                        [Alu_out_MemAddrMux_alu_out].into(),
                    ),
                    PortValue::new(
                        "sel".to_string(),
                        format!("0b{:1b}", self.sel.get_value()),
                        [].into(),
                    ),
                ],
                vec![PortValue::new(
                    "out".to_string(),
                    format!("0x{:X}", self.out.get_value()),
                    [
                        MemAddrMux_out_MemCtl_addr,
                        MemAddrMux_out_RegFileMux_mem_addr,
                    ]
                    .into(),
                )],
            ),
        }
    }
}

impl DatapathComponentDisplayer for MemCtl {
    fn get_datapath_component(&self) -> DatapathComponent {
        DatapathComponent {
            name: "MemCtl".to_string(),
            values: PortValues::new(
                vec![
                    PortValue::new(
                        "addr".to_string(),
                        format!("0x{:X}", self.cpu_addr.get_value()),
                        [MemAddrMux_out_MemCtl_addr].into(),
                    ),
                    PortValue::new(
                        "wdata".to_string(),
                        format!("0x{:X}", self.cpu_wdata.get_value()),
                        [RegFile_rs2_data_MemCtl_wdata].into(),
                    ),
                    PortValue::new(
                        "rmask".to_string(),
                        format!("0b{:4b}", self.cpu_rmask.get_value()),
                        [].into(),
                    ),
                    PortValue::new(
                        "wmask".to_string(),
                        format!("0x{:4b}", self.cpu_wmask.get_value()),
                        [].into(),
                    ),
                ],
                vec![
                    PortValue::new(
                        "rdata".to_string(),
                        format!("0x{:X}", self.cpu_rdata.get_value()),
                        [MemCtl_rdata_Ir_data, MemCtl_rdata_RegFileMux_m_rdata].into(),
                    ),
                    PortValue::new(
                        "resp".to_string(),
                        format!("0b{:1b}", self.cpu_resp.get_value()),
                        [].into(),
                    ),
                ],
            ),
        }
    }
}

impl DatapathComponentDisplayer for Pc {
    fn get_datapath_component(&self) -> DatapathComponent {
        DatapathComponent {
            name: "PC".to_string(),
            values: PortValues::new(
                vec![
                    PortValue::new(
                        "data".to_string(),
                        format!("0x{:X}", self.data.get_value()),
                        [PcMux_out_Pc_data].into(),
                    ),
                    PortValue::new(
                        "load".to_string(),
                        format!("0x{:X}", self.load.get_value()),
                        [].into(),
                    ),
                ],
                vec![PortValue::new(
                    "out".to_string(),
                    format!("0x{:X}", self.out.get_value()),
                    [
                        Pc_out_MemAddrMux_pc,
                        Pc_out_AluMux1_pc,
                        Pc_out_PcMux_pc,
                        Pc_out_RegFileMux_pc,
                    ]
                    .into(),
                )],
            ),
        }
    }
}

impl DatapathComponentDisplayer for PcMux {
    fn get_datapath_component(&self) -> DatapathComponent {
        DatapathComponent {
            name: "PC Mux".to_string(),
            values: PortValues::new(
                vec![
                    PortValue::new(
                        "pc".to_string(),
                        format!("0x{:X}", self.pc.get_value()),
                        [Pc_out_PcMux_pc].into(),
                    ),
                    PortValue::new(
                        "alu_out".to_string(),
                        format!("0x{:X}", self.alu_out.get_value()),
                        [Alu_out_PcMux_alu_out].into(),
                    ),
                    PortValue::new(
                        "sel".to_string(),
                        format!("0b{:1b}", self.sel.get_value()),
                        [].into(),
                    ),
                ],
                vec![PortValue::new(
                    "out".to_string(),
                    format!("0x{:X}", self.out.get_value()),
                    [PcMux_out_Pc_data].into(),
                )],
            ),
        }
    }
}

impl DatapathComponentDisplayer for RegFile {
    fn get_datapath_component(&self) -> DatapathComponent {
        DatapathComponent {
            name: "RegFile".to_string(),
            values: PortValues::new(
                vec![
                    PortValue::new(
                        "rs1_idx".to_string(),
                        format!("0b{:5b}", self.rs1_idx.get_value()),
                        [Ir_rs1_idx_RegFile_rs1_idx].into(),
                    ),
                    PortValue::new(
                        "rs2_idx".to_string(),
                        format!("0b{:5b}", self.rs2_idx.get_value()),
                        [Ir_rs2_idx_RegFile_rs2_idx].into(),
                    ),
                    PortValue::new(
                        "rd_idx".to_string(),
                        format!("0b{:5b}", self.rd_idx.get_value()),
                        [Ir_rd_idx_RegFile_rd_idx].into(),
                    ),
                    PortValue::new(
                        "rd_dat".to_string(),
                        format!("0x{:X}", self.rd_data.get_value()),
                        [RegFileMux_out_RegFile_rd_data].into(),
                    ),
                    PortValue::new(
                        "rd_wr".to_string(),
                        format!("0b{:1b}", self.rd_wr.get_value()),
                        [].into(),
                    ),
                ],
                vec![
                    PortValue::new(
                        "rs1_dat".to_string(),
                        format!("0x{:X}", self.rs1_data.get_value()),
                        [RegFile_rs1_data_Cmp_a, RegFile_rs1_data_AluMux1_rs1_data].into(),
                    ),
                    PortValue::new(
                        "rs2_dat".to_string(),
                        format!("0x{:X}", self.rs2_data.get_value()),
                        [
                            RegFile_rs2_data_CmpMux_rs2_data,
                            RegFile_rs2_data_AluMux2_rs2_data,
                            RegFile_rs2_data_MemCtl_wdata,
                        ]
                        .into(),
                    ),
                ],
            ),
        }
    }
}

impl DatapathComponentDisplayer for RegFileMux {
    fn get_datapath_component(&self) -> DatapathComponent {
        DatapathComponent {
            name: "RegFile Mux".to_string(),
            values: PortValues::new(
                vec![
                    PortValue::new(
                        "alu_out".to_string(),
                        format!("0x{:X}", self.alu_out.get_value()),
                        [Alu_out_RegFileMux_Alu_out].into(),
                    ),
                    PortValue::new(
                        "cmp_out".to_string(),
                        format!("0x{:X}", self.cmp_out.get_value()),
                        [Cmp_out_RegFileMux_Cmp_out].into(),
                    ),
                    PortValue::new(
                        "u_imm".to_string(),
                        format!("0x{:X}", self.u_imm.get_value()),
                        [Ir_imm_RegFileMux_u_imm].into(),
                    ),
                    PortValue::new(
                        "m_addr".to_string(),
                        format!("0x{:X}", self.mem_addr_mux_out.get_value()),
                        [MemAddrMux_out_RegFileMux_mem_addr].into(),
                    ),
                    PortValue::new(
                        "m_rdata".to_string(),
                        format!("0x{:X}", self.mem_rdata.get_value()),
                        [MemCtl_rdata_RegFileMux_m_rdata].into(),
                    ),
                    PortValue::new(
                        "pc".to_string(),
                        format!("0x{:X}", self.pc.get_value()),
                        [Pc_out_RegFileMux_pc].into(),
                    ),
                    PortValue::new(
                        "sel".to_string(),
                        format!("0b{:3b}", self.sel.get_value()),
                        [].into(),
                    ),
                ],
                vec![PortValue::new(
                    "out".to_string(),
                    format!("0x{:X}", self.out.get_value()),
                    [RegFileMux_out_RegFile_rd_data].into(),
                )],
            ),
        }
    }
}
