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
pub struct Mar {
    pub data_inner: Word,
}

impl Mar {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        load: Rx<Byte>,
        data: Rx<Word>,
        out: Tx<Word>,
    ) -> Self {
        let clock_channel = unbounded();
        Mar {
            data_inner: Default::default(),
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
        self.data_inner = Default::default();
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

impl Debug for Mar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MAR: {{data_inner: {:?}, load: {:?}}}",
            self.data_inner,
            self.load.get_value()
        )
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
pub struct MarMux {}

impl MarMux {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        pc: Rx<Word>,
        alu_out: Rx<Word>,
        sel: Rx<Byte>,
        out: Tx<Word>,
    ) -> Self {
        MarMux {
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
            Some(mux_sel::mar::PC_OUT) => self.pc.get_value(),
            Some(mux_sel::mar::ALU_OUT) => self.alu_out.get_value(),
            _ => Word::unknown(),
        };

        self.out.send(out, 0);
    }
}

impl Debug for MarMux {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MARMUX: {{pc: {:?}, alu_out: {:?}, sel: {:?}}}",
            self.pc.get_value(),
            self.alu_out.get_value(),
            self.sel.get_value()
        )
    }
}
