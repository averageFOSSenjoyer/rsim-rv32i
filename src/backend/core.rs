use crate::backend::component::alu::Alu;
use crate::backend::component::alu::AluMux1;
use crate::backend::component::alu::AluMux2;
use crate::backend::component::cmp::Cmp;
use crate::backend::component::cmp::CmpMux;
use crate::backend::component::control::Control;
use crate::backend::component::ir::IR;
use crate::backend::component::mar::Mar;
use crate::backend::component::mar::MarMux;
use crate::backend::component::mem_ctl::{KeyboardMmioCtl, MemCtl, MmioCtl, VgaMmioCtl};
use crate::backend::component::mrdr::Mrdr;
use crate::backend::component::mwdr::Mwdr;
use crate::backend::component::pc::Pc;
use crate::backend::component::pc::PcMux;
use crate::backend::component::regfile::RegFile;
use crate::backend::component::regfile::RegFileMux;
use crate::backend::core::StatsType::InstructionsRan;
use crate::backend::util::byte::Bytes;
use crate::backend::util::types::Byte;
use crate::backend::util::types::States;
use crate::backend::util::types::Word;
use crossbeam_channel::{unbounded, Receiver, Sender};
use rsim_core::component::Component;
use rsim_core::sim_dispatcher::SimDispatcher;
use rsim_core::sim_manager::SimManager;
use rsim_core::tx::Tx;
use rsim_core::types::EventId;
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[cfg(not(target_arch = "wasm32"))]
use std::thread::JoinHandle;
#[cfg(target_arch = "wasm32")]
use wasm_thread as thread;
#[cfg(target_arch = "wasm32")]
use wasm_thread::JoinHandle as JoinHandle;

use strum::EnumIter;
use strum::IntoEnumIterator;
use crate::frontend::core_gui_wrapper::CoreGuiWrapper;

#[derive(EnumIter, Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum StatsType {
    InstructionsRan,
}

#[derive(EnumIter, Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentType {
    Alu,
    AluMux1,
    AluMux2,
    Cmp,
    CmpMux,
    Ir,
    Mar,
    MarMux,
    MemCtl,
    MrDR,
    MwDR,
    Pc,
    PcMux,
    RegFile,
    RegFileMux,
}

/// A wrapper for all the components
#[allow(dead_code)]
pub struct Core {
    ack_channel: (Sender<EventId>, Receiver<EventId>),
    pub sim_manager: Arc<SimManager>,
    pub sim_dispatcher_handlers: Vec<JoinHandle<()>>,
    // todo, clean this mess up, with enum?
    pub mem_ctl: Arc<Mutex<MemCtl>>,
    pub control: Arc<Mutex<Control>>,
    pub ir: Arc<Mutex<IR>>,
    pub pc_mux: Arc<Mutex<PcMux>>,
    pub pc: Arc<Mutex<Pc>>,
    pub mar_mux: Arc<Mutex<MarMux>>,
    pub mar: Arc<Mutex<Mar>>,
    pub mrdr: Arc<Mutex<Mrdr>>,
    pub alu_mux1: Arc<Mutex<AluMux1>>,
    pub alu_mux2: Arc<Mutex<AluMux2>>,
    pub alu: Arc<Mutex<Alu>>,
    pub cmp_mux: Arc<Mutex<CmpMux>>,
    pub cmp: Arc<Mutex<Cmp>>,
    pub regfile_mux: Arc<Mutex<RegFileMux>>,
    pub regfile: Arc<Mutex<RegFile>>,
    pub mwdr: Arc<Mutex<Mwdr>>,
    pub keyboard_mmio_ctl: Arc<Mutex<KeyboardMmioCtl>>,
    pub vga_mmio_ctl: Arc<Mutex<VgaMmioCtl>>,
    commit_file: Mutex<Option<File>>,
    stats: Mutex<HashMap<StatsType, u128>>,
}

impl Core {
    fn log_commits(&self) {
        if let Some(commit_file) = self.commit_file.lock().unwrap().as_mut() {
            // locking is fine here, we are not advancing the sim
            let control = self.control.lock().unwrap();
            let pc = self.pc.lock().unwrap();
            let ir = self.ir.lock().unwrap();
            let regfile = self.regfile.lock().unwrap();
            let mar = self.mar.lock().unwrap();
            let mem_ctl = self.mem_ctl.lock().unwrap();

            if control.state == States::Fetch1
                || control.state == States::Fetch2
                || control.state == States::Fetch3
                || control.state == States::Decode
                || control.state == States::Store1
                || control.state == States::Load1
                || control.state == States::AddrCalc
            {
                return;
            }

            let mut line = String::new();

            line.push_str(&format!(
                "core   0: 3 0x{:x} (0x{:x})",
                pc.data_inner, ir.data_inner
            ));

            if regfile.rd_wr.get_value().is_something_nonzero()
                && ir.get_rd_idx().is_something_nonzero()
            {
                let raw_rd: u8 = Into::<Option<u8>>::into(ir.get_rd_idx()).unwrap();
                if raw_rd < 10 {
                    line.push_str(&format!(" x{}  ", raw_rd))
                } else {
                    line.push_str(&format!(" x{} ", raw_rd))
                }
                line.push_str(&format!("0x{:x}", regfile.rd_data.get_value()));
            }

            if control.state == States::Load2 && control.get_rmask().is_something_nonzero() {
                let rmask = Into::<Option<u8>>::into(control.get_rmask()).unwrap();
                let mut byte_shift = 0;
                for i in 0..4u8 {
                    if (rmask >> i) & 0x1 == 0x1 {
                        byte_shift = i;
                        break;
                    }
                }
                line.push_str(&format!(
                    " mem 0x{:x}",
                    (mar.data_inner & Word::from(0xFFFFFFFCu32)) + Byte::from(byte_shift)
                ));
            }

            if control.state == States::Store2 && control.get_wmask().is_something_nonzero() {
                let wmask = Into::<Option<u8>>::into(control.get_wmask()).unwrap();
                let mut byte_shift = 0;
                for i in 0..4u8 {
                    if (wmask >> i) & 0x1 == 0x1 {
                        byte_shift = i;
                        break;
                    }
                }
                let mut byte_count = 0;
                for i in 0..4u8 {
                    if (wmask >> i) & 0x1 == 0x1 {
                        byte_count += 1;
                    }
                }

                line.push_str(&format!(
                    " mem 0x{:x}",
                    (mar.data_inner & Word::from(0xFFFFFFFCu32)) + Byte::from(byte_shift)
                ));
                if let Some(mwdr) = Into::<Option<u32>>::into(mem_ctl.cpu_wdata.get_value()) {
                    let shifted_data = mwdr >> (8 * byte_shift);
                    let mwdr_str = match byte_count {
                        1 => {
                            format!("{:x}", Byte::from(shifted_data as u8))
                        }
                        2 => {
                            format!("{:x}", Bytes::<2>::from(shifted_data as u16))
                        }
                        4 => {
                            format!("{:x}", Word::from(shifted_data))
                        }
                        _ => "".to_string(),
                    };
                    line.push_str(&format!(" 0x{}", mwdr_str));
                }
            }

            line.push('\n');
            commit_file.write_all(line.as_bytes()).unwrap();

            let instructions_ran = self.stats.lock().unwrap()[&InstructionsRan];
            if instructions_ran % 1000 == 0 {
                println!("commit #{}", instructions_ran);
                print!("{}", line);
            }
        }
    }

    pub fn run_cycle<F: Fn(&CoreGuiWrapper) + Copy>(&self, hook: Option<F>, core_gui_wrapper: Option<&CoreGuiWrapper>) {
        self.sim_manager.run_cycle().unwrap();
        self.sim_manager.run_cycle_end().unwrap();
        self.log_commits();
        if let (Some(ref hook), Some(core_gui_wrapper)) = (hook, core_gui_wrapper) {
            hook(core_gui_wrapper);
        }
    }

    pub fn run_instruction<F: Fn(&CoreGuiWrapper) + Copy>(&self, hook: Option<F>, core_gui_wrapper: Option<&CoreGuiWrapper>) {
        let old_pc = self.pc.lock().unwrap().data_inner;

        while !self.ir.lock().unwrap().can_end() && old_pc == self.pc.lock().unwrap().data_inner {
            self.run_cycle(hook, core_gui_wrapper)
        }

        let mut locked_stats = self.stats.lock().unwrap();
        let instructions_ran = locked_stats[&InstructionsRan];
        locked_stats.insert(InstructionsRan, instructions_ran + 1);
    }

    pub fn run_until_addr<F: Fn(&CoreGuiWrapper) + Copy>(&self, addr: &BTreeSet<Word>, hook: Option<F>, core_gui_wrapper: Option<&CoreGuiWrapper>) {
        while !self.ir.lock().unwrap().can_end() {
            if addr.contains(&self.pc.lock().unwrap().data_inner) {
                break;
            }
            self.run_instruction(hook, core_gui_wrapper);
        }
    }

    pub fn run_end<F: Fn(&CoreGuiWrapper) + Copy>(&self, hook: Option<F>, core_gui_wrapper: Option<&CoreGuiWrapper>) {
        while !self.ir.lock().unwrap().can_end() {
            self.run_instruction(hook, core_gui_wrapper);
        }
    }

    pub fn load_bin(&self, data: &[u8], addr: Word) {
        self.mem_ctl.lock().unwrap().load_bin(data, addr);
    }

    pub fn reset(&self) {
        self.mem_ctl.lock().unwrap().reset();
        self.control.lock().unwrap().reset();
        self.ir.lock().unwrap().reset();
        self.pc_mux.lock().unwrap().reset();
        self.pc.lock().unwrap().reset();
        self.mar_mux.lock().unwrap().reset();
        self.mar.lock().unwrap().reset();
        self.mrdr.lock().unwrap().reset();
        self.alu_mux1.lock().unwrap().reset();
        self.alu_mux2.lock().unwrap().reset();
        self.alu.lock().unwrap().reset();
        self.cmp_mux.lock().unwrap().reset();
        self.cmp.lock().unwrap().reset();
        self.regfile_mux.lock().unwrap().reset();
        self.regfile.lock().unwrap().reset();
        self.mwdr.lock().unwrap().reset();
        self.keyboard_mmio_ctl.lock().unwrap().reset();
        self.vga_mmio_ctl.lock().unwrap().reset();

        self.mem_ctl.lock().unwrap().install_mmio_ctl(
            KeyboardMmioCtl::STATUS_ADDR..KeyboardMmioCtl::DATA_ADDR + 1,
            self.keyboard_mmio_ctl.clone(),
        );
        self.mem_ctl.lock().unwrap().install_mmio_ctl(
            VgaMmioCtl::BASE_ADDR..VgaMmioCtl::BASE_ADDR + VgaMmioCtl::NUM_BYTES as u32,
            self.vga_mmio_ctl.clone(),
        );
    }

    pub fn new(threads_to_use: usize, commit_file: Option<File>) -> Self {
        let ack_channel = unbounded();
        let sim_manager = SimManager::new(ack_channel.1.clone());
        let stats: Mutex<HashMap<StatsType, u128>> = Mutex::new(Default::default());

        for stats_type in StatsType::iter() {
            stats.lock().unwrap().insert(stats_type, 0u128);
        }

        // i swear there has to be a better way of doing this
        let mut mem_ctl_cpu_rdata = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut mem_ctl_cpu_resp = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_mar_load = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_mrdr_load = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_pc_load = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_ir_load = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_rf_load = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_mwdr_load = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_alu_op = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_cmp_op = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_pc_mux_sel = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_alu_mux1_sel = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_alu_mux2_sel = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_rf_mux_sel = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_mar_mux_sel = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_cmp_mux_sel = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_mem_read = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_mem_write = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_mem_rmask = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut control_mem_wmask = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut ir_funct3 = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut ir_funct7 = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut ir_opcode = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut ir_i_imm = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut ir_s_imm = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut ir_b_imm = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut ir_u_imm = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut ir_j_imm = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut ir_rs1_idx = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut ir_rs2_idx = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut ir_rd_idx = Tx::<Byte>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut pc_mux_out = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut pc_out = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut mar_mux_out = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut mar_out = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut mrdr_out = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut alu_mux1_out = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut alu_mux2_out = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut alu_out = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut cmp_mux_out = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut cmp_out = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut regfile_mux_out = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut regfile_rs1_data = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut regfile_rs2_data = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());
        let mut mwdr_out = Tx::<Word>::new(sim_manager.clone(), ack_channel.0.clone());

        let mem_ctl_cpu_rdata_rx = mem_ctl_cpu_rdata.add_rx();
        let mem_ctl_cpu_resp_rx = mem_ctl_cpu_resp.add_rx();
        let control_mar_load_rx = control_mar_load.add_rx();
        let control_mrdr_load_rx = control_mrdr_load.add_rx();
        let control_pc_load_rx = control_pc_load.add_rx();
        let control_ir_load_rx = control_ir_load.add_rx();
        let control_rf_load_rx = control_rf_load.add_rx();
        let control_mwdr_load_rx = control_mwdr_load.add_rx();
        let control_alu_op_rx = control_alu_op.add_rx();
        let control_cmp_op_rx = control_cmp_op.add_rx();
        let control_pc_mux_sel_rx = control_pc_mux_sel.add_rx();
        let control_alu_mux1_sel_rx = control_alu_mux1_sel.add_rx();
        let control_alu_mux2_sel_rx = control_alu_mux2_sel.add_rx();
        let control_rf_mux_sel_rx = control_rf_mux_sel.add_rx();
        let control_mar_mux_sel_rx = control_mar_mux_sel.add_rx();
        let control_cmp_mux_sel_rx = control_cmp_mux_sel.add_rx();
        let control_mem_read_rx = control_mem_read.add_rx();
        let control_mem_write_rx = control_mem_write.add_rx();
        let control_mem_rmask_rx = control_mem_rmask.add_rx();
        let control_mem_wmask_rx = control_mem_wmask.add_rx();
        let ir_funct3_rx = ir_funct3.add_rx();
        let ir_funct7_rx = ir_funct7.add_rx();
        let ir_opcode_rx = ir_opcode.add_rx();
        let ir_i_imm_rx_alu_mux2 = ir_i_imm.add_rx();
        let ir_i_imm_rx_cmp_mux = ir_i_imm.add_rx();
        let ir_s_imm_rx = ir_s_imm.add_rx();
        let ir_b_imm_rx = ir_b_imm.add_rx();
        let ir_u_imm_rx_alu_mux2 = ir_u_imm.add_rx();
        let ir_u_imm_rx_rf_mux = ir_u_imm.add_rx();
        let ir_j_imm_rx = ir_j_imm.add_rx();
        let ir_rs1_idx_rx = ir_rs1_idx.add_rx();
        let ir_rs2_idx_rx = ir_rs2_idx.add_rx();
        let ir_rd_idx_rx = ir_rd_idx.add_rx();
        let pc_mux_out_rx = pc_mux_out.add_rx();
        let pc_out_rx_mar_mux = pc_out.add_rx();
        let pc_out_rx_pc_mux = pc_out.add_rx();
        let pc_out_rx_alu_mux1 = pc_out.add_rx();
        let pc_out_rx_rf_mux = pc_out.add_rx();
        let mar_mux_out_rx = mar_mux_out.add_rx();
        let mar_out_rx_mem_ctl = mar_out.add_rx();
        let mar_out_rx_control = mar_out.add_rx();
        let mar_out_rx_rf_mux = mar_out.add_rx();
        let mar_out_rx_mwdr = mar_out.add_rx();
        let mrdr_out_rx_ir = mrdr_out.add_rx();
        let mrdr_out_rx_rf_mux = mrdr_out.add_rx();
        let alu_mux1_out_rx = alu_mux1_out.add_rx();
        let alu_mux2_out_rx = alu_mux2_out.add_rx();
        let alu_out_rx_pc_mux = alu_out.add_rx();
        let alu_out_rx_mar_mux = alu_out.add_rx();
        let alu_out_rx_rf_mux = alu_out.add_rx();
        let cmp_mux_out_rx = cmp_mux_out.add_rx();
        let cmp_out_rx_control = cmp_out.add_rx();
        let cmp_out_rx_rf_mux = cmp_out.add_rx();
        let regfile_mux_out_rx = regfile_mux_out.add_rx();
        let regfile_rs1_data_rx_alu_mux1 = regfile_rs1_data.add_rx();
        let regfile_rs1_data_rx_cmp = regfile_rs1_data.add_rx();
        let regfile_rs2_data_rx_alu_mux2 = regfile_rs2_data.add_rx();
        let regfile_rs2_data_rx_cmp_mux = regfile_rs2_data.add_rx();
        let regfile_rs2_data_rx_mwdr = regfile_rs2_data.add_rx();
        let mwdr_out_rx = mwdr_out.add_rx();

        let mem_ctl = Arc::new(Mutex::new(MemCtl::new(
            0,
            sim_manager.clone(),
            ack_channel.0.clone(),
            mar_out_rx_mem_ctl,
            mwdr_out_rx,
            control_mem_read_rx,
            control_mem_rmask_rx,
            control_mem_write_rx,
            control_mem_wmask_rx,
            mem_ctl_cpu_rdata,
            mem_ctl_cpu_resp,
        )));
        let keyboard_mmio_ctl = Arc::new(Mutex::new(KeyboardMmioCtl::new()));
        mem_ctl.lock().unwrap().install_mmio_ctl(
            KeyboardMmioCtl::STATUS_ADDR..KeyboardMmioCtl::DATA_ADDR + 1,
            keyboard_mmio_ctl.clone(),
        );
        let vga_mmio_ctl = Arc::new(Mutex::new(VgaMmioCtl::new()));
        mem_ctl.lock().unwrap().install_mmio_ctl(
            VgaMmioCtl::BASE_ADDR..VgaMmioCtl::BASE_ADDR + VgaMmioCtl::NUM_BYTES as u32,
            vga_mmio_ctl.clone(),
        );

        let control = Arc::new(Mutex::new(Control::new(
            1,
            sim_manager.clone(),
            ack_channel.0.clone(),
            ir_funct3_rx,
            ir_funct7_rx,
            cmp_out_rx_control,
            ir_opcode_rx,
            mar_out_rx_control,
            mem_ctl_cpu_resp_rx,
            control_mar_load,
            control_mrdr_load,
            control_pc_load,
            control_ir_load,
            control_rf_load,
            control_mwdr_load,
            control_alu_op,
            control_cmp_op,
            control_pc_mux_sel,
            control_alu_mux1_sel,
            control_alu_mux2_sel,
            control_rf_mux_sel,
            control_mar_mux_sel,
            control_cmp_mux_sel,
            control_mem_read,
            control_mem_write,
            control_mem_rmask,
            control_mem_wmask,
        )));

        let ir = Arc::new(Mutex::new(IR::new(
            2,
            sim_manager.clone(),
            ack_channel.0.clone(),
            control_ir_load_rx,
            mrdr_out_rx_ir,
            ir_funct3,
            ir_funct7,
            ir_opcode,
            ir_i_imm,
            ir_s_imm,
            ir_b_imm,
            ir_u_imm,
            ir_j_imm,
            ir_rs1_idx,
            ir_rs2_idx,
            ir_rd_idx,
        )));

        let pc_mux = Arc::new(Mutex::new(PcMux::new(
            3,
            sim_manager.clone(),
            ack_channel.0.clone(),
            pc_out_rx_pc_mux,
            alu_out_rx_pc_mux,
            control_pc_mux_sel_rx,
            pc_mux_out,
        )));

        let pc = Arc::new(Mutex::new(Pc::new(
            4,
            sim_manager.clone(),
            ack_channel.0.clone(),
            control_pc_load_rx,
            pc_mux_out_rx,
            pc_out,
        )));

        let mar_mux = Arc::new(Mutex::new(MarMux::new(
            5,
            sim_manager.clone(),
            ack_channel.0.clone(),
            pc_out_rx_mar_mux,
            alu_out_rx_mar_mux,
            control_mar_mux_sel_rx,
            mar_mux_out,
        )));

        let mar = Arc::new(Mutex::new(Mar::new(
            6,
            sim_manager.clone(),
            ack_channel.0.clone(),
            control_mar_load_rx,
            mar_mux_out_rx,
            mar_out,
        )));

        let mrdr = Arc::new(Mutex::new(Mrdr::new(
            7,
            sim_manager.clone(),
            ack_channel.0.clone(),
            control_mrdr_load_rx,
            mem_ctl_cpu_rdata_rx,
            mrdr_out,
        )));

        let alu_mux1 = Arc::new(Mutex::new(AluMux1::new(
            8,
            sim_manager.clone(),
            ack_channel.0.clone(),
            regfile_rs1_data_rx_alu_mux1,
            pc_out_rx_alu_mux1,
            control_alu_mux1_sel_rx,
            alu_mux1_out,
        )));

        let alu_mux2 = Arc::new(Mutex::new(AluMux2::new(
            9,
            sim_manager.clone(),
            ack_channel.0.clone(),
            ir_i_imm_rx_alu_mux2,
            ir_u_imm_rx_alu_mux2,
            ir_b_imm_rx,
            ir_s_imm_rx,
            ir_j_imm_rx,
            regfile_rs2_data_rx_alu_mux2,
            control_alu_mux2_sel_rx,
            alu_mux2_out,
        )));

        let alu = Arc::new(Mutex::new(Alu::new(
            10,
            sim_manager.clone(),
            ack_channel.0.clone(),
            alu_mux1_out_rx,
            alu_mux2_out_rx,
            control_alu_op_rx,
            alu_out,
        )));

        let cmp_mux = Arc::new(Mutex::new(CmpMux::new(
            11,
            sim_manager.clone(),
            ack_channel.0.clone(),
            regfile_rs2_data_rx_cmp_mux,
            ir_i_imm_rx_cmp_mux,
            control_cmp_mux_sel_rx,
            cmp_mux_out,
        )));

        let cmp = Arc::new(Mutex::new(Cmp::new(
            12,
            sim_manager.clone(),
            ack_channel.0.clone(),
            regfile_rs1_data_rx_cmp,
            cmp_mux_out_rx,
            control_cmp_op_rx,
            cmp_out,
        )));

        let regfile_mux = Arc::new(Mutex::new(RegFileMux::new(
            13,
            sim_manager.clone(),
            ack_channel.0.clone(),
            alu_out_rx_rf_mux,
            cmp_out_rx_rf_mux,
            ir_u_imm_rx_rf_mux,
            mar_out_rx_rf_mux,
            mrdr_out_rx_rf_mux,
            pc_out_rx_rf_mux,
            control_rf_mux_sel_rx,
            regfile_mux_out,
        )));

        let regfile = Arc::new(Mutex::new(RegFile::new(
            14,
            sim_manager.clone(),
            ack_channel.0.clone(),
            ir_rs1_idx_rx,
            ir_rs2_idx_rx,
            control_rf_load_rx,
            ir_rd_idx_rx,
            regfile_mux_out_rx,
            regfile_rs1_data,
            regfile_rs2_data,
        )));

        let mwdr = Arc::new(Mutex::new(Mwdr::new(
            15,
            sim_manager.clone(),
            ack_channel.0.clone(),
            control_mwdr_load_rx,
            mar_out_rx_mwdr,
            regfile_rs2_data_rx_mwdr,
            mwdr_out,
        )));

        let components_vec: Vec<Arc<Mutex<dyn Component>>> = vec![
            mem_ctl.clone(),
            control.clone(),
            ir.clone(),
            pc_mux.clone(),
            pc.clone(),
            mar_mux.clone(),
            mar.clone(),
            mrdr.clone(),
            alu_mux1.clone(),
            alu_mux2.clone(),
            alu.clone(),
            cmp_mux.clone(),
            cmp.clone(),
            regfile_mux.clone(),
            regfile.clone(),
            mwdr.clone(),
        ];

        let sim_dispatchers: Vec<_> = components_vec
            .chunks((components_vec.len() as f32 / threads_to_use as f32).ceil() as usize)
            .map(|component| SimDispatcher::new(Arc::downgrade(&sim_manager), component.into()))
            .collect();
        sim_dispatchers.iter().for_each(|s| s.init());

        sim_manager.register_do_not_end(0);

        let mut sim_dispatcher_handlers = vec![];
        for sim_dispatcher in sim_dispatchers {
            sim_dispatcher_handlers.push(thread::spawn(move || sim_dispatcher.run()));
        }

        Core {
            ack_channel,
            sim_manager,
            sim_dispatcher_handlers,
            mem_ctl,
            control,
            ir,
            pc_mux,
            pc,
            mar_mux,
            mar,
            mrdr,
            alu_mux1,
            alu_mux2,
            alu,
            cmp_mux,
            cmp,
            regfile_mux,
            regfile,
            keyboard_mmio_ctl,
            vga_mmio_ctl,
            commit_file: Mutex::new(commit_file),
            mwdr,
            stats,
        }
    }
}