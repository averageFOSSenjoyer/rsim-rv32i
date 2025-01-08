use crate::backend::util::byte::{ByteOrd, SignedOrd};
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
use std::cmp::Ordering::Less;
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
pub struct Cmp {}

impl Cmp {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        a: Rx<Word>,
        b: Rx<Word>,
        op: Rx<Byte>,
        out: Tx<Word>,
    ) -> Self {
        Cmp {
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

        if match self.op.get_value().into() {
            Some(funct3::branch::BEQ) => a == b,
            Some(funct3::branch::BNE) => a != b,
            Some(funct3::branch::BLT) => a.signed_cmp(b) == Less,
            Some(funct3::branch::BGE) => a.signed_cmp(b) != Less,
            Some(funct3::branch::BLTU) => a.byte_cmp(b) == Less,
            Some(funct3::branch::BGEU) => a.byte_cmp(b) != Less,
            _ => false,
        } {
            self.out.send(Word::from(1u32), 0);
        } else {
            self.out.send(Word::from(0u32), 0);
        };
    }
}

impl Debug for Cmp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CMP: {{a: {:?}, b: {:?}, op: {:?}}}",
            self.a.get_value(),
            self.b.get_value(),
            self.op.get_value()
        )
    }
}

#[ComponentAttribute({
"port": {
    "input": [
        ["rs2", "Word"],
        ["i_imm", "Word"],
        ["sel", "Byte"]
    ],
    "output": [
        ["out", "Word"]
    ]
}
})]
pub struct CmpMux {}

impl CmpMux {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        rs2: Rx<Word>,
        i_imm: Rx<Word>,
        sel: Rx<Byte>,
        out: Tx<Word>,
    ) -> Self {
        CmpMux {
            component_id,
            sim_manager,
            ack_sender,
            rs2,
            i_imm,
            sel,
            out,
        }
    }

    fn init_impl(&mut self) {}

    fn reset_impl(&mut self) {}

    fn poll_impl(&mut self) {}

    fn on_comb(&mut self) {
        let out = match self.sel.get_value().into() {
            Some(mux_sel::cmp::RS2_OUT) => self.rs2.get_value(),
            Some(mux_sel::cmp::I_IMM) => self.i_imm.get_value(),
            _ => Word::unknown(),
        };

        self.out.send(out, 0);
    }
}
