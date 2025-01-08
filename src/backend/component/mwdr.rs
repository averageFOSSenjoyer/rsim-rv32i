use crate::backend::util::types::Byte;
use crate::backend::util::types::Word;
use crossbeam_channel::{unbounded, Sender};
use rsim_core::component::Component;
use rsim_core::rx::Rx;
use rsim_core::sim_manager::SimManager;
use rsim_core::tx::Tx;
use rsim_core::types::Input;
use rsim_core::types::{ComponentId, EventId, Output};
use rsim_macro::ComponentAttribute;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

#[ComponentAttribute({
"port": {
    "input": [
        ["load", "Byte"],
        ["mar", "Word"],
        ["rs2_data", "Word"]
    ],
    "output": [
        ["out", "Word"]
    ],
    "clock": true
}
})]
pub struct Mwdr {
    pub data_inner: Word,
}

impl Mwdr {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        load: Rx<Byte>,
        mar: Rx<Word>,
        rs2_data: Rx<Word>,
        out: Tx<Word>,
    ) -> Self {
        let clock_channel = unbounded();
        Mwdr {
            data_inner: Default::default(),
            component_id,
            sim_manager,
            ack_sender,
            clock_sender: clock_channel.0,
            clock_receiver: clock_channel.1,
            load,
            mar,
            rs2_data,
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
            self.data_inner = self.rs2_data.get_value();
        }
    }

    fn on_comb(&mut self) {
        if let Some(mar) = Into::<Option<u32>>::into(self.mar.get_value()) {
            self.out
                .send(self.data_inner << Word::from(8 * (mar & 0x3)), 0);
        }
    }
}

impl Debug for Mwdr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MwDR: {{inner: {:?}, rs2_data: {:?}, mar: {:?}}}",
            self.data_inner,
            self.rs2_data.get_value(),
            self.mar.get_value()
        )
    }
}
