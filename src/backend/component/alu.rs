use crate::backend::util::byte::Shra;
use crate::backend::util::types::Word;
use crate::backend::util::types::*;
use crossbeam_channel::Sender;
use rsim_core::component::Component;
use rsim_core::rx::Rx;
use rsim_core::sim_manager::SimManager;
use rsim_core::tx::Tx;
use rsim_core::types::ComponentId;
use rsim_core::types::EventId;
use rsim_macro::ComponentAttribute;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

#[ComponentAttribute({
"port": {
    "input": [
        ["a", "Word"],
        ["b", "Word"],
        ["op", "Byte"]
    ],
    "output": [
        ["out", "Word"]
    ]
}
})]
pub struct Alu {}

impl Alu {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        a: Rx<Word>,
        b: Rx<Word>,
        op: Rx<Byte>,
        out: Tx<Word>,
    ) -> Self {
        Alu {
            component_id,
            sim_manager,
            ack_sender,
            a,
            b,
            op,
            out,
        }
    }
    fn init_impl(&mut self) {}

    fn reset_impl(&mut self) {}

    fn poll_impl(&mut self) {}

    fn on_comb(&mut self) {
        let a = self.a.get_value();
        let b = self.b.get_value();
        let op = self.op.get_value();

        let out = match Into::<Option<u8>>::into(op) {
            Some(alu_op::ADD) => a + b,
            Some(alu_op::SLL) => a << (b & Word::from(0x1Fu32)),
            Some(alu_op::SRA) => a.shra(b & Word::from(0x1Fu32)),
            Some(alu_op::SUB) => a - b,
            Some(alu_op::XOR) => a ^ b,
            Some(alu_op::SRL) => a >> (b & Word::from(0x1Fu32)),
            Some(alu_op::OR) => a | b,
            Some(alu_op::AND) => a & b,
            _ => Word::unknown(),
        };

        self.out.send(out, 0);
    }
}

impl Debug for Alu {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Alu: {{a: {:?}, b:{:?}, op: {:?} }}",
            self.a.get_value(),
            self.b.get_value(),
            self.op.get_value()
        )
    }
}

#[ComponentAttribute({
"port": {
    "input": [
        ["rs1", "Word"],
        ["pc", "Word"],
        ["sel", "Byte"]
    ],
    "output": [
        ["out", "Word"]
    ]
}
})]
pub struct AluMux1 {}

impl AluMux1 {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        rs1: Rx<Word>,
        pc: Rx<Word>,
        sel: Rx<Byte>,
        out: Tx<Word>,
    ) -> Self {
        AluMux1 {
            component_id,
            sim_manager,
            ack_sender,
            rs1,
            pc,
            sel,
            out,
        }
    }

    fn init_impl(&mut self) {}

    fn reset_impl(&mut self) {}

    fn poll_impl(&mut self) {}

    fn on_comb(&mut self) {
        let out = match self.sel.get_value().into() {
            Some(mux_sel::alu1::RS1_OUT) => self.rs1.get_value(),
            Some(mux_sel::alu1::PC_OUT) => self.pc.get_value(),
            _ => Word::unknown(),
        };

        self.out.send(out, 0);
    }
}

#[ComponentAttribute({
"port": {
    "input": [
        ["i_imm", "Word"],
        ["u_imm", "Word"],
        ["b_imm", "Word"],
        ["s_imm", "Word"],
        ["j_imm", "Word"],
        ["rs2", "Word"],
        ["sel", "Byte"]
    ],
    "output": [
        ["out", "Word"]
    ]
}
})]
pub struct AluMux2 {}

impl AluMux2 {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        i_imm: Rx<Word>,
        u_imm: Rx<Word>,
        b_imm: Rx<Word>,
        s_imm: Rx<Word>,
        j_imm: Rx<Word>,
        rs2: Rx<Word>,
        sel: Rx<Byte>,
        out: Tx<Word>,
    ) -> Self {
        AluMux2 {
            component_id,
            sim_manager,
            ack_sender,
            i_imm,
            u_imm,
            b_imm,
            s_imm,
            j_imm,
            rs2,
            sel,
            out,
        }
    }

    fn init_impl(&mut self) {}

    fn reset_impl(&mut self) {}

    fn poll_impl(&mut self) {}

    fn on_comb(&mut self) {
        let out = match self.sel.get_value().into() {
            Some(mux_sel::alu2::I_IMM) => self.i_imm.get_value(),
            Some(mux_sel::alu2::U_IMM) => self.u_imm.get_value(),
            Some(mux_sel::alu2::B_IMM) => self.b_imm.get_value(),
            Some(mux_sel::alu2::S_IMM) => self.s_imm.get_value(),
            Some(mux_sel::alu2::J_IMM) => self.j_imm.get_value(),
            Some(mux_sel::alu2::RS2_OUT) => self.rs2.get_value(),
            _ => Word::unknown(),
        };

        self.out.send(out, 0);
    }
}
