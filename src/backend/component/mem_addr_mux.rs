use crate::backend::util::types::Byte;
use crate::backend::util::types::{Word, mux_sel};
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
        ["pc", "Word"],
        ["alu_out", "Word"],
        ["sel", "Byte"]
    ],
    "output": [
        ["out", "Word"]
    ]
}
})]
pub struct MemAddrMux {}

impl MemAddrMux {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        pc: Rx<Word>,
        alu_out: Rx<Word>,
        sel: Rx<Byte>,
        out: Tx<Word>,
    ) -> Self {
        MemAddrMux {
            component_id,
            sim_manager,
            ack_sender,
            pc,
            alu_out,
            sel,
            out,
        }
    }

    fn init_impl(&mut self) {}

    fn reset_impl(&mut self) {}

    fn poll_impl(&mut self) {}

    fn on_comb(&mut self) {
        let out = match self.sel.get_value().into() {
            Some(mux_sel::mem_addr::PC_OUT) => self.pc.get_value(),
            Some(mux_sel::mem_addr::ALU_OUT) => self.alu_out.get_value(),
            _ => Word::unknown(),
        };

        self.out.send(out, 0);
    }
}

impl Debug for MemAddrMux {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MEM_ADDR_MUX: {{pc: {:?}, alu_out: {:?}, sel: {:?}}}",
            self.pc.get_value(),
            self.alu_out.get_value(),
            self.sel.get_value()
        )
    }
}
