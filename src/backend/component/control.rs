use crate::backend::util::types::States::*;
use crate::backend::util::types::*;
use crossbeam_channel::{Sender, unbounded};
use rsim_core::component::Component;
use rsim_core::rx::Rx;
use rsim_core::sim_manager::SimManager;
use rsim_core::tx::Tx;
use rsim_core::types::ComponentId;
use rsim_core::types::EventId;
use rsim_core::types::Input;
use rsim_core::types::Output;
use rsim_macro::ComponentAttribute;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

#[ComponentAttribute({
"port": {
    "input": [
        ["funct3", "Byte"],
        ["funct7", "Byte"],
        ["cmp_out", "Word"],
        ["opcode", "Byte"],
        ["mar", "Word"],
        ["mem_resp", "Byte"]
    ],
    "output": [
        ["load_mar", "Byte"],
        ["load_mrdr", "Byte"],
        ["load_pc", "Byte"],
        ["load_ir", "Byte"],
        ["load_regfile", "Byte"],
        ["load_mwdr", "Byte"],
        ["alu_op", "Byte"],
        ["cmp_op", "Byte"],
        ["pc_mux_sel", "Byte"],
        ["alu_mux1_sel", "Byte"],
        ["alu_mux2_sel", "Byte"],
        ["regfile_mux_sel", "Byte"],
        ["mar_mux_sel", "Byte"],
        ["cmp_mux_sel", "Byte"],
        ["mem_read", "Byte"],
        ["mem_write", "Byte"],
        ["mem_rmask", "Byte"],
        ["mem_wmask", "Byte"]
    ],
    "clock": true
}
})]
pub struct Control {
    pub state: States,
    pub next_state: States,
}

impl Control {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        funct3: Rx<Byte>,
        funct7: Rx<Byte>,
        cmp_out: Rx<Word>,
        opcode: Rx<Byte>,
        mar: Rx<Word>,
        mem_resp: Rx<Byte>,
        load_mar: Tx<Byte>,
        load_mrdr: Tx<Byte>,
        load_pc: Tx<Byte>,
        load_ir: Tx<Byte>,
        load_regfile: Tx<Byte>,
        load_mwdr: Tx<Byte>,
        alu_op: Tx<Byte>,
        cmp_op: Tx<Byte>,
        pc_mux_sel: Tx<Byte>,
        alu_mux1_sel: Tx<Byte>,
        alu_mux2_sel: Tx<Byte>,
        regfile_mux_sel: Tx<Byte>,
        mar_mux_sel: Tx<Byte>,
        cmp_mux_sel: Tx<Byte>,
        mem_read: Tx<Byte>,
        mem_write: Tx<Byte>,
        mem_rmask: Tx<Byte>,
        mem_wmask: Tx<Byte>,
    ) -> Self {
        let clock_channel = unbounded();

        Control {
            state: Fetch1,
            next_state: Fetch1,
            component_id,
            sim_manager,
            ack_sender,
            clock_sender: clock_channel.0,
            clock_receiver: clock_channel.1,
            funct3,
            funct7,
            cmp_out,
            opcode,
            mar,
            mem_resp,
            load_mar,
            load_mrdr,
            load_pc,
            load_ir,
            load_regfile,
            load_mwdr,
            alu_op,
            cmp_op,
            pc_mux_sel,
            alu_mux1_sel,
            alu_mux2_sel,
            regfile_mux_sel,
            mar_mux_sel,
            cmp_mux_sel,
            mem_read,
            mem_write,
            mem_rmask,
            mem_wmask,
        }
    }

    fn init_impl(&mut self) {}

    fn reset_impl(&mut self) {
        self.state = Fetch1;
        self.next_state = Fetch1;
    }

    fn poll_impl(&mut self) {}

    fn on_clock(&mut self) {
        self.state = self.next_state;
    }

    pub fn get_rmask(&self) -> Byte {
        if self.state == Fetch2 {
            Byte::from(0x0Fu8)
        } else {
            match self.funct3.get_value().into() {
                Some(funct3::load::LW) => Byte::from(0x0Fu8),
                Some(funct3::load::LH) | Some(funct3::load::LHU) => {
                    Byte::from(0x03u8) << (self.mar.get_value() & Byte::from(0x3u8))
                }
                Some(funct3::load::LB) | Some(funct3::load::LBU) => {
                    Byte::from(0x01u8) << (self.mar.get_value() & Byte::from(0x3u8))
                }
                _ => Byte::unknown(),
            }
        }
    }

    pub fn get_wmask(&self) -> Byte {
        match self.funct3.get_value().into() {
            Some(funct3::store::SW) => Byte::from(0x0Fu8),
            Some(funct3::store::SH) => {
                Byte::from(0x03u8) << (self.mar.get_value() & Byte::from(0x3u8))
            }
            Some(funct3::store::SB) => {
                Byte::from(0x01u8) << (self.mar.get_value() & Byte::from(0x3u8))
            }
            _ => Byte::unknown(),
        }
    }

    fn set_default_control_signals(&mut self) {
        self.load_mar.send(Byte::from(0u8), 0);
        self.load_mrdr.send(Byte::from(0u8), 0);
        self.load_pc.send(Byte::from(0u8), 0);
        self.load_ir.send(Byte::from(0u8), 0);
        self.load_regfile.send(Byte::from(0u8), 0);
        self.load_mwdr.send(Byte::from(0u8), 0);
        self.mem_read.send(Byte::from(0u8), 0);
        self.mem_write.send(Byte::from(0u8), 0);
        self.mem_wmask.send(self.get_wmask(), 0);
        self.mem_rmask.send(self.get_rmask(), 0);
    }

    fn load_pc(&mut self, sel: u8) {
        self.load_pc.send(Byte::from(1u8), 0);
        self.pc_mux_sel.send(Byte::from(sel), 0);
    }

    fn load_regfile(&mut self, sel: u8) {
        self.load_regfile.send(Byte::from(1u8), 0);
        self.regfile_mux_sel.send(Byte::from(sel), 0);
    }

    fn load_mar(&mut self, sel: u8) {
        self.load_mar.send(Byte::from(1u8), 0);
        self.mar_mux_sel.send(Byte::from(sel), 0);
    }

    fn load_ir(&mut self) {
        self.load_ir.send(Byte::from(1u8), 0);
    }

    fn load_dout(&mut self) {
        self.load_mwdr.send(Byte::from(1u8), 0);
    }

    fn set_alu(&mut self, sel1: u8, sel2: u8, alu_op: u8) {
        self.alu_mux1_sel.send(Byte::from(sel1), 0);
        self.alu_mux2_sel.send(Byte::from(sel2), 0);
        self.alu_op.send(Byte::from(alu_op), 0);
    }

    fn set_cmp(&mut self, sel: u8, cmp_op: u8) {
        self.cmp_mux_sel.send(Byte::from(sel), 0);
        self.cmp_op.send(Byte::from(cmp_op), 0);
    }

    fn read_from_mem(&mut self) {
        self.load_mrdr.send(Byte::from(1u8), 0);
        self.mem_read.send(Byte::from(1u8), 0);
    }

    fn write_to_mem(&mut self) {
        self.mem_write.send(Byte::from(1u8), 0);
    }

    fn set_control_signal(&mut self) {
        match self.state {
            Fetch1 => {
                self.load_mar(mux_sel::mar::PC_OUT);
            }
            Fetch2 => {
                self.read_from_mem();
            }
            Fetch3 => {
                self.load_ir();
            }
            Decode => {}
            Imm => {
                if let (Some(funct3), Some(funct7)) = (
                    Into::<Option<u8>>::into(self.funct3.get_value()),
                    Into::<Option<u8>>::into(self.funct7.get_value()),
                ) {
                    match funct3 {
                        funct3::arith::SLT => {
                            self.load_regfile(mux_sel::regfile::BR_EN);
                            self.set_cmp(mux_sel::cmp::I_IMM, funct3::branch::BLT);
                        }
                        funct3::arith::SLTU => {
                            self.load_regfile(mux_sel::regfile::BR_EN);
                            self.set_cmp(mux_sel::cmp::I_IMM, funct3::branch::BLTU);
                        }
                        funct3::arith::SR => {
                            self.load_regfile(mux_sel::regfile::ALU_OUT);
                            self.set_alu(
                                mux_sel::alu1::RS1_OUT,
                                mux_sel::alu2::I_IMM,
                                if (funct7 >> 5) & 0x1 == 0x1 {
                                    alu_op::SRA
                                } else {
                                    alu_op::SRL
                                },
                            );
                        }
                        _ => {
                            self.load_regfile(mux_sel::regfile::ALU_OUT);
                            self.set_alu(mux_sel::alu1::RS1_OUT, mux_sel::alu2::I_IMM, funct3);
                        }
                    }
                }
                self.load_pc(mux_sel::pc::PC_PLUS4);
            }
            Reg => {
                if let (Some(funct3), Some(funct7)) = (
                    Into::<Option<u8>>::into(self.funct3.get_value()),
                    Into::<Option<u8>>::into(self.funct7.get_value()),
                ) {
                    match funct3 {
                        funct3::arith::ADD => {
                            self.load_regfile(mux_sel::regfile::ALU_OUT);
                            self.set_alu(
                                mux_sel::alu1::RS1_OUT,
                                mux_sel::alu2::RS2_OUT,
                                if (funct7 >> 5) & 0x1 == 0x1 {
                                    alu_op::SUB
                                } else {
                                    alu_op::ADD
                                },
                            );
                        }
                        funct3::arith::SR => {
                            self.load_regfile(mux_sel::regfile::ALU_OUT);
                            self.set_alu(
                                mux_sel::alu1::RS1_OUT,
                                mux_sel::alu2::RS2_OUT,
                                if (funct7 >> 5) & 0x1 == 0x1 {
                                    alu_op::SRA
                                } else {
                                    alu_op::SRL
                                },
                            );
                        }
                        funct3::arith::SLT => {
                            self.load_regfile(mux_sel::regfile::BR_EN);
                            self.set_cmp(mux_sel::cmp::RS2_OUT, funct3::branch::BLT);
                        }
                        funct3::arith::SLTU => {
                            self.load_regfile(mux_sel::regfile::BR_EN);
                            self.set_cmp(mux_sel::cmp::RS2_OUT, funct3::branch::BLTU);
                        }
                        _ => {
                            self.load_regfile(mux_sel::regfile::ALU_OUT);
                            self.set_alu(mux_sel::alu1::RS1_OUT, mux_sel::alu2::RS2_OUT, funct3);
                        }
                    }
                }
                self.load_pc(mux_sel::pc::PC_PLUS4);
            }
            Lui => {
                self.load_regfile(mux_sel::regfile::U_IMM);
                self.load_pc(mux_sel::pc::PC_PLUS4);
            }
            Br => {
                self.set_alu(mux_sel::alu1::PC_OUT, mux_sel::alu2::B_IMM, alu_op::ADD);
                if let Some(funct3) = Into::<Option<u8>>::into(self.funct3.get_value()) {
                    self.set_cmp(mux_sel::cmp::RS2_OUT, funct3);
                }
                self.load_pc(if self.cmp_out.get_value().is_something_nonzero() {
                    mux_sel::pc::ALU_OUT
                } else {
                    mux_sel::pc::PC_PLUS4
                });
            }
            Auipc => {
                self.load_regfile(mux_sel::regfile::ALU_OUT);
                self.set_alu(mux_sel::alu1::PC_OUT, mux_sel::alu2::U_IMM, alu_op::ADD);
                if let Some(funct3) = Into::<Option<u8>>::into(self.funct3.get_value()) {
                    self.set_cmp(mux_sel::cmp::RS2_OUT, funct3);
                }
                self.load_pc(mux_sel::pc::PC_PLUS4);
            }
            AddrCalc => {
                self.load_mar(mux_sel::mar::ALU_OUT);
                self.set_alu(
                    mux_sel::alu1::RS1_OUT,
                    if self.opcode.get_value() == Byte::from(opcode::LOAD) {
                        mux_sel::alu2::I_IMM
                    } else {
                        mux_sel::alu2::S_IMM
                    },
                    alu_op::ADD,
                );
                if self.opcode.get_value() == Byte::from(opcode::STORE) {
                    self.load_dout();
                }
            }
            Load1 => {
                self.read_from_mem();
            }
            Load2 => {
                if let Some(funct3) = Into::<Option<u8>>::into(self.funct3.get_value()) {
                    match funct3 {
                        funct3::load::LB => {
                            self.load_regfile(mux_sel::regfile::LB);
                        }
                        funct3::load::LH => {
                            self.load_regfile(mux_sel::regfile::LH);
                        }
                        funct3::load::LW => {
                            self.load_regfile(mux_sel::regfile::LW);
                        }
                        funct3::load::LBU => {
                            self.load_regfile(mux_sel::regfile::LBU);
                        }
                        funct3::load::LHU => {
                            self.load_regfile(mux_sel::regfile::LHU);
                        }
                        _ => {}
                    }
                }
                self.load_pc(mux_sel::pc::PC_PLUS4);
            }
            Store1 => {
                self.write_to_mem();
            }
            Store2 => {
                self.load_pc(mux_sel::pc::PC_PLUS4);
            }
            Jal => {
                self.load_pc(mux_sel::pc::ALU_OUT);
                self.set_alu(mux_sel::alu1::PC_OUT, mux_sel::alu2::J_IMM, alu_op::ADD);
                self.load_regfile(mux_sel::regfile::PC_PLUS4);
            }
            Jalr => {
                self.load_pc(mux_sel::pc::ALU_MOD2);
                self.set_alu(mux_sel::alu1::RS1_OUT, mux_sel::alu2::I_IMM, alu_op::ADD);
                self.load_regfile(mux_sel::regfile::PC_PLUS4);
            }
        }
    }

    fn set_next_state(&mut self) {
        self.next_state = self.state;

        self.next_state = match self.state {
            Fetch1 => Fetch2,
            Fetch2 => {
                if self.mem_resp.get_value().is_something_nonzero() {
                    Fetch3
                } else {
                    Fetch2
                }
            }
            Fetch3 => Decode,
            Decode => match Into::<Option<u8>>::into(self.opcode.get_value()) {
                Some(opcode::LUI) => Lui,
                Some(opcode::AUIPC) => Auipc,
                Some(opcode::JAL) => Jal,
                Some(opcode::JALR) => Jalr,
                Some(opcode::BR) => Br,
                Some(opcode::LOAD) | Some(opcode::STORE) => AddrCalc,
                Some(opcode::IMM) => Imm,
                Some(opcode::REG) => Reg,
                _ => Fetch1,
            },
            AddrCalc => {
                if self.opcode.get_value() == Byte::from(opcode::LOAD) {
                    Load1
                } else {
                    Store1
                }
            }
            Load1 => {
                if self.mem_resp.get_value().is_something_nonzero() {
                    Load2
                } else {
                    Load1
                }
            }
            Store1 => {
                if self.mem_resp.get_value().is_something_nonzero() {
                    Store2
                } else {
                    Store1
                }
            }
            _ => Fetch1,
        }
    }

    fn on_comb(&mut self) {
        self.set_default_control_signals();
        self.set_control_signal();
        self.set_next_state();
    }
}

impl Debug for Control {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Control {{State: {:?}, funct3: {:?}}}",
            self.state,
            self.funct3.get_value()
        )
    }
}
