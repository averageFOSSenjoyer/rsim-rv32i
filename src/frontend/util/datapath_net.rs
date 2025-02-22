use crate::backend::core::ComponentType;
use crate::backend::core::ComponentType::{
    Alu, AluMux1, AluMux2, Cmp, CmpMux, Ir, MemAddrMux, MemCtl, Pc, PcMux, RegFile, RegFileMux,
};
use crate::frontend::util::datapath_component::GLOBAL_FRAME_WIDTH;
use crate::frontend::util::datapath_net::DatapathNet::*;
use egui::{Pos2, Vec2};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, EnumIter, Eq, PartialEq, Hash)]
#[allow(non_camel_case_types)]
pub enum DatapathNet {
    MemCtl_rdata_Ir_data,
    MemCtl_rdata_RegFileMux_m_rdata,
    RegFileMux_out_RegFile_rd_data,
    Ir_rs1_idx_RegFile_rs1_idx,
    Ir_rs2_idx_RegFile_rs2_idx,
    Ir_rd_idx_RegFile_rd_idx,
    Ir_u_imm_RegFileMux_u_imm,
    Ir_i_imm_AluMux2_i_imm,
    Ir_i_imm_CmpMux_i_imm,
    Ir_u_imm_AluMux2_u_imm,
    Ir_b_imm_AluMux2_b_imm,
    Ir_s_imm_AluMux2_s_imm,
    Ir_j_imm_AluMux2_j_imm,
    AluMux2_out_Alu_b,
    AluMux1_out_Alu_a,
    RegFile_rs1_data_AluMux1_rs1_data,
    RegFile_rs1_data_Cmp_a,
    RegFile_rs2_data_AluMux2_rs2_data,
    RegFile_rs2_data_CmpMux_rs2_data,
    RegFile_rs2_data_MemCtl_wdata,
    Pc_out_AluMux1_pc,
    Pc_out_RegFileMux_pc,
    Pc_out_MemAddrMux_pc,
    Pc_out_PcMux_pc,
    PcMux_out_Pc_data,
    MemAddrMux_out_RegFileMux_mem_addr,
    MemAddrMux_out_MemCtl_addr,
    CmpMux_out_Cmp_b,
    Cmp_out_RegFileMux_Cmp_out,
    Alu_out_RegFileMux_Alu_out,
    Alu_out_MemAddrMux_alu_out,
    Alu_out_PcMux_alu_out,
}

pub trait DatapathNetDispalyer {
    fn get_nth_port_pos(&self, component: &ComponentType, n: u32, is_input_port: bool) -> Pos2;
    fn get_points(&self) -> Vec<Pos2>;
}

impl DatapathNetDispalyer for DatapathNet {
    fn get_nth_port_pos(&self, component: &ComponentType, n: u32, is_input_port: bool) -> Pos2 {
        let mut pos2 = component.get_frame_offset();

        if !is_input_port {
            pos2.x += component.get_frame_size().x;
        }
        pos2.y += 42.0;
        pos2.y += n as f32 * 21.0; // this number arrived to me in sleep
        pos2
    }

    // if it works it works
    fn get_points(&self) -> Vec<Pos2> {
        match *self {
            MemCtl_rdata_Ir_data => {
                vec![
                    self.get_nth_port_pos(&MemCtl, 0, false),
                    self.get_nth_port_pos(&Ir, 0, true),
                ]
            }
            MemCtl_rdata_RegFileMux_m_rdata => {
                let start = self.get_nth_port_pos(&MemCtl, 0, false);
                let end = self.get_nth_port_pos(&RegFileMux, 4, true);
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    start + Vec2::new(10.0, -80.0),
                    start + Vec2::new(10.0 + GLOBAL_FRAME_WIDTH + 100.0, -80.0),
                    end + Vec2::new(-30.0, 0.0),
                    end,
                ]
            }
            RegFileMux_out_RegFile_rd_data => {
                let start = self.get_nth_port_pos(&RegFileMux, 0, false);
                let end = self.get_nth_port_pos(&RegFile, 3, true);
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    end + Vec2::new(-40.0, 0.0),
                    end,
                ]
            }
            Ir_rs1_idx_RegFile_rs1_idx => {
                let start = self.get_nth_port_pos(&Ir, 5, false);
                let end = self.get_nth_port_pos(&RegFile, 0, true);
                let mid = Pos2::new(
                    (start + Vec2::new(60.0, 0.0)).x,
                    Ir.get_frame_offset().y - 30.0,
                );
                vec![
                    start,
                    start + Vec2::new(60.0, 0.0),
                    mid,
                    mid + Vec2::new(60.0 + GLOBAL_FRAME_WIDTH + 40.0, 0.0),
                    end + Vec2::new(-10.0, 0.0),
                    end,
                ]
            }
            Ir_rs2_idx_RegFile_rs2_idx => {
                let start = self.get_nth_port_pos(&Ir, 6, false);
                let end = self.get_nth_port_pos(&RegFile, 1, true);
                let mid = Pos2::new(
                    (start + Vec2::new(70.0, 0.0)).x,
                    Ir.get_frame_offset().y - 20.0,
                );
                vec![
                    start,
                    start + Vec2::new(70.0, 0.0),
                    mid,
                    mid + Vec2::new(50.0 + GLOBAL_FRAME_WIDTH + 30.0, 0.0),
                    end + Vec2::new(-20.0, 0.0),
                    end,
                ]
            }
            Ir_rd_idx_RegFile_rd_idx => {
                let start = self.get_nth_port_pos(&Ir, 7, false);
                let end = self.get_nth_port_pos(&RegFile, 2, true);
                let mid = Pos2::new(
                    (start + Vec2::new(80.0, 0.0)).x,
                    &Ir.get_frame_offset().y - 10.0,
                );
                vec![
                    start,
                    start + Vec2::new(80.0, 0.0),
                    mid,
                    mid + Vec2::new(40.0 + GLOBAL_FRAME_WIDTH + 20.0, 0.0),
                    end + Vec2::new(-30.0, 0.0),
                    end,
                ]
            }
            Ir_u_imm_RegFileMux_u_imm => {
                let start = self.get_nth_port_pos(&Ir, 1, false);
                let end = self.get_nth_port_pos(&RegFileMux, 2, true);
                vec![
                    start,
                    start + Vec2::new(100.0, 0.0),
                    end + Vec2::new(-20.0, 0.0),
                    end,
                ]
            }
            Ir_i_imm_AluMux2_i_imm => {
                let start = self.get_nth_port_pos(&Ir, 0, false);
                let end = self.get_nth_port_pos(&AluMux2, 0, true);
                let mid = Pos2::new(
                    (start + Vec2::new(10.0, 0.0)).x,
                    Ir.get_frame_offset().y - 90.0,
                );
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    mid,
                    mid + Vec2::new(
                        110.0 + GLOBAL_FRAME_WIDTH + 50.0 + GLOBAL_FRAME_WIDTH + 90.0,
                        0.0,
                    ),
                    end + Vec2::new(-10.0, 0.0),
                    end,
                ]
            }
            Ir_i_imm_CmpMux_i_imm => {
                let start = self.get_nth_port_pos(&Ir, 0, false);
                let end = self.get_nth_port_pos(&CmpMux, 1, true);
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    start + Vec2::new(10.0, 200.0),
                    start + Vec2::new(40.0, 200.0),
                    end + Vec2::new(-450.0, 0.0),
                    end,
                ]
            }
            Ir_u_imm_AluMux2_u_imm => {
                let start = self.get_nth_port_pos(&Ir, 1, false);
                let end = self.get_nth_port_pos(&AluMux2, 1, true);
                let mid = Pos2::new(
                    (start + Vec2::new(20.0, 0.0)).x,
                    Ir.get_frame_offset().y - 80.0,
                );
                vec![
                    start,
                    start + Vec2::new(20.0, 0.0),
                    mid,
                    mid + Vec2::new(
                        100.0 + GLOBAL_FRAME_WIDTH + 50.0 + GLOBAL_FRAME_WIDTH + 80.0,
                        0.0,
                    ),
                    end + Vec2::new(-20.0, 0.0),
                    end,
                ]
            }
            Ir_b_imm_AluMux2_b_imm => {
                let start = self.get_nth_port_pos(&Ir, 2, false);
                let end = self.get_nth_port_pos(&AluMux2, 2, true);
                let mid = Pos2::new(
                    (start + Vec2::new(30.0, 0.0)).x,
                    Ir.get_frame_offset().y - 70.0,
                );
                vec![
                    start,
                    start + Vec2::new(30.0, 0.0),
                    mid,
                    mid + Vec2::new(
                        90.0 + GLOBAL_FRAME_WIDTH + 50.0 + GLOBAL_FRAME_WIDTH + 70.0,
                        0.0,
                    ),
                    end + Vec2::new(-30.0, 0.0),
                    end,
                ]
            }
            Ir_s_imm_AluMux2_s_imm => {
                let start = self.get_nth_port_pos(&Ir, 3, false);
                let end = self.get_nth_port_pos(&AluMux2, 3, true);
                let mid = Pos2::new(
                    (start + Vec2::new(40.0, 0.0)).x,
                    Ir.get_frame_offset().y - 60.0,
                );
                vec![
                    start,
                    start + Vec2::new(40.0, 0.0),
                    mid,
                    mid + Vec2::new(
                        80.0 + GLOBAL_FRAME_WIDTH + 50.0 + GLOBAL_FRAME_WIDTH + 60.0,
                        0.0,
                    ),
                    end + Vec2::new(-40.0, 0.0),
                    end,
                ]
            }
            Ir_j_imm_AluMux2_j_imm => {
                let start = self.get_nth_port_pos(&Ir, 4, false);
                let end = self.get_nth_port_pos(&AluMux2, 4, true);
                let mid = Pos2::new(
                    (start + Vec2::new(50.0, 0.0)).x,
                    Ir.get_frame_offset().y - 50.0,
                );
                vec![
                    start,
                    start + Vec2::new(50.0, 0.0),
                    mid,
                    mid + Vec2::new(
                        70.0 + GLOBAL_FRAME_WIDTH + 50.0 + GLOBAL_FRAME_WIDTH + 50.0,
                        0.0,
                    ),
                    end + Vec2::new(-50.0, 0.0),
                    end,
                ]
            }
            AluMux2_out_Alu_b => {
                vec![
                    self.get_nth_port_pos(&AluMux2, 0, false),
                    self.get_nth_port_pos(&Alu, 0, true),
                ]
            }
            AluMux1_out_Alu_a => {
                let start = self.get_nth_port_pos(&AluMux1, 0, false);
                let end = self.get_nth_port_pos(&Alu, 1, true);
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    end + Vec2::new(-10.0, 0.0),
                    end,
                ]
            }
            RegFile_rs1_data_AluMux1_rs1_data => {
                let start = self.get_nth_port_pos(&RegFile, 0, false);
                let end = self.get_nth_port_pos(&AluMux1, 0, true);
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    end + Vec2::new(-90.0, 0.0),
                    end,
                ]
            }
            RegFile_rs1_data_Cmp_a => {
                let start = self.get_nth_port_pos(&RegFile, 0, false);
                let end = self.get_nth_port_pos(&Cmp, 0, true);
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    end + Vec2::new(-220.0, 0.0),
                    end,
                ]
            }
            RegFile_rs2_data_AluMux2_rs2_data => {
                let start = self.get_nth_port_pos(&RegFile, 1, false);
                let end = self.get_nth_port_pos(&AluMux2, 5, true);
                vec![
                    start,
                    start + Vec2::new(30.0, 0.0),
                    end + Vec2::new(-70.0, 0.0),
                    end,
                ]
            }
            RegFile_rs2_data_CmpMux_rs2_data => {
                let start = self.get_nth_port_pos(&RegFile, 1, false);
                let end = self.get_nth_port_pos(&CmpMux, 0, true);
                vec![
                    start,
                    start + Vec2::new(30.0, 0.0),
                    end + Vec2::new(-70.0, 0.0),
                    end,
                ]
            }
            RegFile_rs2_data_MemCtl_wdata => {
                let start = self.get_nth_port_pos(&RegFile, 1, false);
                let end = self.get_nth_port_pos(&MemCtl, 1, true);
                vec![
                    start,
                    start + Vec2::new(30.0, 0.0),
                    start + Vec2::new(30.0, 220.0),
                    end + Vec2::new(-10.0, 220.0),
                    end + Vec2::new(-10.0, 0.0),
                    end,
                ]
            }
            Pc_out_AluMux1_pc => {
                let start = self.get_nth_port_pos(&Pc, 0, false);
                let end = self.get_nth_port_pos(&AluMux1, 1, true);
                vec![
                    start,
                    start + Vec2::new(410.0, 0.0),
                    end + Vec2::new(-80.0, 0.0),
                    end,
                ]
            }
            Pc_out_RegFileMux_pc => {
                let start = self.get_nth_port_pos(&Pc, 0, false);
                let end = self.get_nth_port_pos(&RegFileMux, 5, true);
                vec![
                    start,
                    start + Vec2::new(90.0, 0.0),
                    end + Vec2::new(-30.0, 0.0),
                    end,
                ]
            }
            Pc_out_MemAddrMux_pc => {
                let start = self.get_nth_port_pos(&Pc, 0, false);
                let end = self.get_nth_port_pos(&MemAddrMux, 0, true);
                let mid = Pos2::new(
                    (start + Vec2::new(10.0, 0.0)).x,
                    Pc.get_frame_offset().y - 10.0,
                );
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    mid,
                    mid + Vec2::new(-260.0, 0.0),
                    end + Vec2::new(-10.0, 0.0),
                    end,
                ]
            }
            Pc_out_PcMux_pc => {
                let start = self.get_nth_port_pos(&Pc, 0, false);
                let end = self.get_nth_port_pos(&PcMux, 0, true);
                let mid = Pos2::new(
                    (start + Vec2::new(10.0, 0.0)).x,
                    Pc.get_frame_offset().y - 10.0,
                );
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    mid,
                    mid + Vec2::new(-260.0, 0.0),
                    end + Vec2::new(-10.0, 0.0),
                    end,
                ]
            }
            PcMux_out_Pc_data => {
                vec![
                    self.get_nth_port_pos(&PcMux, 0, false),
                    self.get_nth_port_pos(&Pc, 0, true),
                ]
            }
            MemAddrMux_out_RegFileMux_mem_addr => {
                let start = self.get_nth_port_pos(&MemAddrMux, 0, false);
                let end = self.get_nth_port_pos(&RegFileMux, 3, true);
                vec![
                    start,
                    start + Vec2::new(230.0, 0.0),
                    end + Vec2::new(-20.0, 0.0),
                    end,
                ]
            }
            MemAddrMux_out_MemCtl_addr => {
                let start = self.get_nth_port_pos(&MemAddrMux, 0, false);
                let end = self.get_nth_port_pos(&MemCtl, 0, true);
                let mid = Pos2::new(
                    (start + Vec2::new(10.0, 0.0)).x,
                    MemAddrMux.get_frame_offset().y - 10.0,
                );
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    mid,
                    mid + Vec2::new(-140.0, 0.0),
                    end + Vec2::new(-20.0, 0.0),
                    end,
                ]
            }
            CmpMux_out_Cmp_b => {
                let start = self.get_nth_port_pos(&CmpMux, 0, false);
                let end = self.get_nth_port_pos(&Cmp, 1, true);
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    end + Vec2::new(-10.0, 0.0),
                    end,
                ]
            }
            Cmp_out_RegFileMux_Cmp_out => {
                let start = self.get_nth_port_pos(&Cmp, 0, false);
                let end = self.get_nth_port_pos(&RegFileMux, 1, true);
                let mid = Pos2::new(
                    (start + Vec2::new(10.0, 0.0)).x,
                    Cmp.get_frame_offset().y - 10.0,
                );
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    mid,
                    mid + Vec2::new(-630.0, 0.0),
                    end + Vec2::new(-10.0, 0.0),
                    end,
                ]
            }
            Alu_out_RegFileMux_Alu_out => {
                let start = self.get_nth_port_pos(&Alu, 0, false);
                let end = self.get_nth_port_pos(&RegFileMux, 0, true);
                let mid = Pos2::new(
                    (start + Vec2::new(10.0, 0.0)).x,
                    Alu.get_frame_offset().y - 110.0,
                );
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    mid,
                    mid + Vec2::new(-630.0, 0.0),
                    end + Vec2::new(-10.0, 0.0),
                    end,
                ]
            }
            Alu_out_MemAddrMux_alu_out => {
                let start = self.get_nth_port_pos(&Alu, 0, false);
                let end = self.get_nth_port_pos(&MemAddrMux, 1, true);
                let mid = Pos2::new(
                    (start + Vec2::new(10.0, 0.0)).x,
                    Alu.get_frame_offset().y - 110.0,
                );
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    mid,
                    mid + Vec2::new(-1020.0, 0.0),
                    end + Vec2::new(-40.0, 0.0),
                    end,
                ]
            }
            Alu_out_PcMux_alu_out => {
                let start = self.get_nth_port_pos(&Alu, 0, false);
                let end = self.get_nth_port_pos(&PcMux, 1, true);
                let mid = Pos2::new(
                    (start + Vec2::new(10.0, 0.0)).x,
                    Alu.get_frame_offset().y - 110.0,
                );
                vec![
                    start,
                    start + Vec2::new(10.0, 0.0),
                    mid,
                    mid + Vec2::new(-1020.0, 0.0),
                    end + Vec2::new(-40.0, 0.0),
                    end,
                ]
            }
        }
    }
}
