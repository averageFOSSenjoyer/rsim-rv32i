use crate::backend::util::byte::Bytes;
use crate::backend::util::helper::sign_extend;
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

#[derive(Debug, Clone, Copy)]
pub struct Registers {
    pub data: [Word; 32],
}

impl Registers {
    fn read(&self, index: Bytes<1>) -> Word {
        Into::<Option<u8>>::into(index)
            .map(|idx| {
                if idx != 0 {
                    self.data[idx as usize]
                } else {
                    Word::zeros()
                }
            })
            .unwrap_or(Word::unknown())
    }

    fn write(&mut self, index: Bytes<1>, value: Word) {
        if let Some(idx) = Into::<Option<u8>>::into(index) {
            if idx != 0 {
                self.data[idx as usize] = value
            }
        }
    }

    const fn default_data() -> [Word; 32] {
        cfg_if::cfg_if! {
            if #[cfg(feature = "reset_unknown_regfile")] {
                let mut registers = [Word::unknown(); 32];
                registers[0] = Word::zeros();
                registers
            } else {
                [Word::zeros(); 32]
            }
        }
    }

    fn reset(&mut self) {
        self.data = Registers::default_data();
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            data: Registers::default_data(),
        }
    }
}

/// A registerfile, consists of 32 4-byte registers
#[ComponentAttribute({
"port": {
    "input": [
        ["rs1_idx", "Byte"],
        ["rs2_idx", "Byte"],
        ["rd_wr", "Byte"],
        ["rd_idx", "Byte"],
        ["rd_data", "Word"]
    ],
    "output": [
        ["rs1_data", "Word"],
        ["rs2_data", "Word"]
    ],
    "clock": true
}
})]
pub struct RegFile {
    pub registers: Registers,
}

impl RegFile {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        rs1_idx: Rx<Byte>,
        rs2_idx: Rx<Byte>,
        rd_wr: Rx<Byte>,
        rd_idx: Rx<Byte>,
        rd_data: Rx<Word>,
        rs1_data: Tx<Word>,
        rs2_data: Tx<Word>,
    ) -> Self {
        let clock_channel = unbounded();
        RegFile {
            registers: Default::default(),
            component_id,
            sim_manager,
            ack_sender,
            clock_sender: clock_channel.0,
            clock_receiver: clock_channel.1,
            rs1_idx,
            rs2_idx,
            rd_wr,
            rd_idx,
            rd_data,
            rs1_data,
            rs2_data,
        }
    }

    fn init_impl(&mut self) {}

    fn reset_impl(&mut self) {
        self.registers.reset();
    }

    fn poll_impl(&mut self) {}

    fn on_clock(&mut self) {
        if self.rd_wr.get_value() != Byte::zeros() {
            self.registers
                .write(self.rd_idx.get_value(), self.rd_data.get_value());
        }
    }

    fn on_comb(&mut self) {
        self.rs1_data
            .send(self.registers.read(self.rs1_idx.get_value()), 0);
        self.rs2_data
            .send(self.registers.read(self.rs2_idx.get_value()), 0);
    }
}

impl Debug for RegFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RegFile: {{rd_wr: {:?}, rd_idx: {:?}, rd_data: {:?}, rs1_idx: {:?}, rs2_idx: {:?}, data: {:?}}}",
            self.rd_wr.get_value(),
            self.rd_idx.get_value(),
            self.rd_data.get_value(),
            self.rs1_idx.get_value(),
            self.rs2_idx.get_value(),
            self.registers
        )
    }
}

#[ComponentAttribute({
"port": {
    "input": [
        ["alu_out", "Word"],
        ["cmp_out", "Word"],
        ["u_imm", "Word"],
        ["mem_addr_mux_out", "Word"],
        ["mem_rdata", "Word"],
        ["pc", "Word"],
        ["sel", "Byte"]
    ],
    "output": [
        ["out", "Word"]
    ]
}
})]
pub struct RegFileMux {}

impl RegFileMux {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        alu_out: Rx<Word>,
        cmp_out: Rx<Word>,
        u_imm: Rx<Word>,
        mem_addr_mux_out: Rx<Word>,
        mem_rdata: Rx<Word>,
        pc: Rx<Word>,
        sel: Rx<Byte>,
        out: Tx<Word>,
    ) -> Self {
        RegFileMux {
            component_id,
            sim_manager,
            ack_sender,
            alu_out,
            cmp_out,
            u_imm,
            mem_addr_mux_out,
            mem_rdata,
            pc,
            sel,
            out,
        }
    }

    fn init_impl(&mut self) {}

    fn reset_impl(&mut self) {}

    fn poll_impl(&mut self) {}

    fn on_comb(&mut self) {
        let mem_rdata_idx = (Into::<Option<u32>>::into(self.mem_addr_mux_out.get_value())
            .unwrap_or(0)
            & 0x3u32) as usize;
        let out = match self.sel.get_value().into() {
            Some(mux_sel::regfile::ALU_OUT) => self.alu_out.get_value(),
            Some(mux_sel::regfile::BR_EN) => self.cmp_out.get_value(),
            Some(mux_sel::regfile::U_IMM) => self.u_imm.get_value(),
            Some(mux_sel::regfile::LW) => self.mem_rdata.get_value(),
            Some(mux_sel::regfile::PC_PLUS4) => self.pc.get_value() + Word::from(4u32),
            Some(mux_sel::regfile::LB) => {
                let val = self.mem_rdata.get_value()[mem_rdata_idx]
                    .map(|byte| Word::from(byte as u32))
                    .unwrap_or(Word::unknown());
                if !val.has_unknown() {
                    sign_extend(Into::<Option<u32>>::into(val).unwrap(), 7)
                } else {
                    Word::unknown()
                }
            }
            Some(mux_sel::regfile::LBU) => self.mem_rdata.get_value()[mem_rdata_idx]
                .map(|byte| Word::from(byte as u32))
                .unwrap_or(Word::unknown()),
            Some(mux_sel::regfile::LH) => {
                let val = self.mem_rdata.get_value()[mem_rdata_idx]
                    .map(|lsb| {
                        self.mem_rdata
                            .get_value()
                            .data
                            .get(mem_rdata_idx + 1)
                            .unwrap_or(&None)
                            .map(|msb| Word::from((((msb as u16) << 8) | lsb as u16) as u32))
                            .unwrap_or(Word::unknown())
                    })
                    .unwrap_or(Word::unknown());
                if !val.has_unknown() {
                    sign_extend(Into::<Option<u32>>::into(val).unwrap(), 15)
                } else {
                    Word::unknown()
                }
            }
            Some(mux_sel::regfile::LHU) => self.mem_rdata.get_value()[mem_rdata_idx]
                .map(|lsb| {
                    self.mem_rdata.get_value()[mem_rdata_idx + 1]
                        .map(|msb| Word::from((((msb as u16) << 8) | lsb as u16) as u32))
                        .unwrap_or(Word::unknown())
                })
                .unwrap_or(Word::unknown()),
            _ => Word::unknown(),
        };

        self.out.send(out, 0);
    }
}

impl Debug for RegFileMux {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RegFileMux: {{alu_out: {:?}, cmp_out: {:?}, u_imm: {:?}, mem_addr_mux_out: {:?} mem_rdata: {:?} pc: {:?} sel: {:?}}}",
            self.alu_out.get_value(),
            self.cmp_out.get_value(),
            self.u_imm.get_value(),
            self.mem_addr_mux_out.get_value(),
            self.mem_rdata.get_value(),
            self.pc.get_value(),
            self.sel.get_value()
        )
    }
}
