use crate::backend::util::helper::sign_extend;
use crate::backend::util::types::Byte;
use crate::backend::util::types::Word;
use crossbeam_channel::{unbounded, Sender};
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
        ["load", "Byte"],
        ["data", "Word"]
    ],
    "output": [
        ["funct3", "Byte"],
        ["funct7", "Byte"],
        ["opcode", "Byte"],
        ["i_imm", "Word"],
        ["s_imm", "Word"],
        ["b_imm", "Word"],
        ["u_imm", "Word"],
        ["j_imm", "Word"],
        ["rs1", "Byte"],
        ["rs2", "Byte"],
        ["rd", "Byte"]
    ],
    "clock": true
}
})]
pub struct IR {
    pub data_inner: Word,
}

impl IR {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        load: Rx<Byte>,
        data: Rx<Word>,
        funct3: Tx<Byte>,
        funct7: Tx<Byte>,
        opcode: Tx<Byte>,
        i_imm: Tx<Word>,
        s_imm: Tx<Word>,
        b_imm: Tx<Word>,
        u_imm: Tx<Word>,
        j_imm: Tx<Word>,
        rs1: Tx<Byte>,
        rs2: Tx<Byte>,
        rd: Tx<Byte>,
    ) -> Self {
        let clock_channel = unbounded();

        IR {
            data_inner: Default::default(),
            component_id,
            sim_manager,
            ack_sender,
            clock_sender: clock_channel.0,
            clock_receiver: clock_channel.1,
            load,
            data,
            funct3,
            funct7,
            opcode,
            i_imm,
            s_imm,
            b_imm,
            u_imm,
            j_imm,
            rs1,
            rs2,
            rd,
        }
    }

    fn init_impl(&mut self) {}

    fn reset_impl(&mut self) {
        self.data_inner = Default::default();
    }

    fn poll_impl(&mut self) {}

    fn on_clock(&mut self) {
        if self.load.get_value().is_something_nonzero() {
            self.data_inner = self.data.get_value();
        }
    }

    pub fn get_rd_idx(&self) -> Byte {
        if let Some(inst) = Into::<Option<u32>>::into(self.data_inner) {
            Byte::from(((inst >> 7) & 0x1F) as u8)
        } else {
            Byte::unknown()
        }
    }

    fn on_comb(&mut self) {
        if let Some(inst) = Into::<Option<u32>>::into(self.data_inner) {
            self.funct3
                .send(Byte::from(((inst >> 12) & 0b111) as u8), 0);
            self.funct7
                .send(Byte::from(((inst >> 25) & 0b1111111) as u8), 0);
            self.opcode.send(Byte::from((inst & 0b1111111) as u8), 0);
            self.i_imm.send(sign_extend(inst >> 20, 11), 0);
            self.s_imm.send(
                sign_extend(
                    (((inst >> 25) & 0b1111111) << 5) | ((inst >> 7) & 0b11111),
                    11,
                ),
                0,
            );
            self.b_imm.send(
                sign_extend(
                    (((inst >> 31) & 0b1) << 12)
                        | (((inst >> 7) & 0b1) << 11)
                        | (((inst >> 25) & 0b111111) << 5)
                        | (((inst >> 8) & 0b1111) << 1),
                    12,
                ),
                0,
            );
            self.u_imm
                .send(Word::from(((inst >> 12) & 0xFFFFF) << 12), 0);
            self.j_imm.send(
                sign_extend(
                    (((inst >> 31) & 0b1) << 20)
                        | (((inst >> 12) & 0xFF) << 12)
                        | (((inst >> 20) & 0b1) << 11)
                        | (((inst >> 21) & 0x3FF) << 1),
                    20,
                ),
                0,
            );
            self.rs1.send(Byte::from(((inst >> 15) & 0x1F) as u8), 0);
            self.rs2.send(Byte::from(((inst >> 20) & 0x1F) as u8), 0);
            self.rd.send(Byte::from(((inst >> 7) & 0x1F) as u8), 0);
        }
    }

    pub fn can_end(&self) -> bool {
        self.data_inner == Word::from(0xF0002013u32) || self.data_inner == Word::from(0x00000063u32)
    }
}

impl Debug for IR {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "IR: {{{:?}}}", self.data_inner.clone())
    }
}
