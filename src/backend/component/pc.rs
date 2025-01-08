use crate::backend::util::types::Byte;
use crate::backend::util::types::{mux_sel, Word};
use crossbeam_channel::unbounded;
use crossbeam_channel::Sender;
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
use std::process::exit;
use std::sync::Arc;

#[ComponentAttribute({
"port": {
    "input": [
        ["load", "Byte"],
        ["data", "Word"]
    ],
    "output": [
        ["out", "Word"]
    ],
    "clock": true
}
})]
pub struct Pc {
    pub data_inner: Word,
}

impl Pc {
    const STARTING_PC: u32 = 0x40000000u32;
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        load: Rx<Byte>,
        data: Rx<Word>,
        out: Tx<Word>,
    ) -> Self {
        let clock_channel = unbounded();
        Pc {
            data_inner: Word::from(Self::STARTING_PC),
            component_id,
            sim_manager,
            ack_sender,
            clock_sender: clock_channel.0,
            clock_receiver: clock_channel.1,
            load,
            data,
            out,
        }
    }
    fn init_impl(&mut self) {}

    fn reset_impl(&mut self) {
        self.data_inner = Word::from(Self::STARTING_PC);
    }

    fn poll_impl(&mut self) {}

    fn on_clock(&mut self) {
        if self.load.get_value().is_something_nonzero() {
            self.data_inner = self.data.get_value();
        }
    }

    fn on_comb(&mut self) {
        self.out.send(self.data_inner, 0);
    }
}

impl Debug for Pc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.data_inner.has_unknown() {
            exit(0);
        }
        write!(f, "PC {{{:?}}}", self.data_inner)
    }
}

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
pub struct PcMux {}

impl PcMux {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        pc: Rx<Word>,
        alu_out: Rx<Word>,
        sel: Rx<Byte>,
        out: Tx<Word>,
    ) -> Self {
        PcMux {
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
            Some(mux_sel::pc::PC_PLUS4) => self.pc.get_value() + Word::from(4u32),
            Some(mux_sel::pc::ALU_OUT) => self.alu_out.get_value(),
            Some(mux_sel::pc::ALU_MOD2) => self.alu_out.get_value() & Word::from(0xFFFFFFFEu32),
            _ => Word::unknown(),
        };

        self.out.send(out, 0);
    }
}
