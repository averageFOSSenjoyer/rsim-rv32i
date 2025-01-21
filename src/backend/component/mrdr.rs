use crate::backend::util::types::Byte;
use crate::backend::util::types::Word;
use crossbeam_channel::Sender;
use crossbeam_channel::unbounded;
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
pub struct Mrdr {
    pub data_inner: Word,
}

impl Mrdr {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        load: Rx<Byte>,
        data: Rx<Word>,
        out: Tx<Word>,
    ) -> Self {
        let clock_channel = unbounded();
        Mrdr {
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

impl Debug for Mrdr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MrDR: {{data: {:?}, data_inner: {:?}, load: {:?}}}",
            self.data.get_value(),
            self.data_inner,
            self.load.get_value()
        )
    }
}
