use crate::backend::util::types::Byte;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc};
use crossbeam_channel::{Receiver, Sender};
use crate::backend::component::mem_ctl::VgaMmioCtl;
use crate::backend::core::Core;
use crate::backend::util::types::Word;
use crate::frontend::core_gui_wrapper::ControlCommand::*;
use crate::frontend::tab::datapath::DatapathComponentMap;
use crate::backend::core::ComponentType::*;
use crate::frontend::util::datapath_component::DatapathComponentDisplayer;

/// A wrapper for `crate::frontend::core_app`
/// that communicates with the ui through channels.
///
/// Mostly due to wasm thread's limitation that the main thread cannot block or lock
pub struct CoreGuiWrapper {
    core: Arc<Core>,
    console_vga_buffer_sender: Sender<[u8; VgaMmioCtl::NUM_BYTES]>,
    console_keyboard_buffer_receiver: Receiver<u8>,
    control_command_receiver: Receiver<ControlCommand>,
    control_ack_sender: Sender<()>,
    register_data_sender: Sender<RegisterData>,
    breakpoints_receiver: Receiver<BTreeSet<Word>>,
    breakpoints: BTreeSet<Word>,
    memory_sender: Sender<BTreeMap<Word, Byte>>,
    label_sender: Sender<BTreeMap<Word, String>>,
    load_elf_receiver: Receiver<Vec<u8>>,
    datapath_component_sender: Sender<DatapathComponentMap>,
}

impl CoreGuiWrapper {
    pub fn new(
        core: Arc<Core>,
        console_vga_buffer_sender: Sender<[u8; VgaMmioCtl::NUM_BYTES]>,
        console_keyboard_buffer_receiver: Receiver<u8>,
        control_command_receiver: Receiver<ControlCommand>,
        control_ack_sender: Sender<()>,
        register_data_sender: Sender<RegisterData>,
        breakpoints_receiver: Receiver<BTreeSet<Word>>,
        memory_sender: Sender<BTreeMap<Word, Byte>>,
        label_sender: Sender<BTreeMap<Word, String>>,
        load_elf_receiver: Receiver<Vec<u8>>,
        datapath_component_sender: Sender<DatapathComponentMap>,
    ) -> Self {
        Self {
            core,
            console_vga_buffer_sender,
            console_keyboard_buffer_receiver,
            control_command_receiver,
            control_ack_sender,
            register_data_sender,
            breakpoints_receiver,
            breakpoints: Default::default(),
            memory_sender,
            label_sender,
            load_elf_receiver,
            datapath_component_sender,
        }
    }

    pub fn send_update(&self) {
        self.console_vga_buffer_sender.try_send(
            *self.core.vga_mmio_ctl.lock().unwrap().get_buffer()
        ).unwrap();

        let register_data = RegisterData {
            state: format!(
                "{}",
                self.core.control.lock().unwrap().state.clone()
            ),
            ir: format!(
                "{}",
                self.core.ir.lock().unwrap().data_inner.clone()
            ),
            pc: format!(
                "{}",
                self.core.pc.lock().unwrap().data_inner.clone()
            ),
            mar: format!(
                "{}",
                self.core.mar.lock().unwrap().data_inner.clone()
            ),
            mrdr: format!(
                "{}",
                self.core.mrdr.lock().unwrap().data_inner.clone()
            ),
            mwdr: format!(
                "{}",
                self.core.mwdr.lock().unwrap().data_inner.clone()
            ),
            regfile: self.core.regfile.lock().unwrap().registers.data.map(|byte| byte.to_string()),
        };
        self.register_data_sender.try_send(register_data).unwrap();

        self.memory_sender.try_send(self.core.mem_ctl.lock().unwrap().backend_mem.clone()).unwrap();
        self.label_sender.try_send(self.core.mem_ctl.lock().unwrap().label.clone()).unwrap();

        let mut datapath_components = DatapathComponentMap::default();
        datapath_components.insert(Alu, self.core.alu.lock().unwrap().get_datapath_component());
        datapath_components.insert(AluMux1, self.core.alu_mux1.lock().unwrap().get_datapath_component());
        datapath_components.insert(AluMux2, self.core.alu_mux2.lock().unwrap().get_datapath_component());
        datapath_components.insert(Cmp, self.core.cmp.lock().unwrap().get_datapath_component());
        datapath_components.insert(CmpMux, self.core.cmp_mux.lock().unwrap().get_datapath_component());
        datapath_components.insert(Ir, self.core.ir.lock().unwrap().get_datapath_component());
        datapath_components.insert(Mar, self.core.mar.lock().unwrap().get_datapath_component());
        datapath_components.insert(MarMux, self.core.mar_mux.lock().unwrap().get_datapath_component());
        datapath_components.insert(MemCtl, self.core.mem_ctl.lock().unwrap().get_datapath_component());
        datapath_components.insert(MrDR, self.core.mrdr.lock().unwrap().get_datapath_component());
        datapath_components.insert(MwDR, self.core.mwdr.lock().unwrap().get_datapath_component());
        datapath_components.insert(Pc, self.core.pc.lock().unwrap().get_datapath_component());
        datapath_components.insert(PcMux, self.core.pc_mux.lock().unwrap().get_datapath_component());
        datapath_components.insert(RegFile, self.core.regfile.lock().unwrap().get_datapath_component());
        datapath_components.insert(RegFileMux, self.core.regfile_mux.lock().unwrap().get_datapath_component());

        self.datapath_component_sender.try_send(datapath_components).unwrap();
    }

    pub fn event_loop(&mut self) {
        loop {
            while let Ok(breakpoint) = self.breakpoints_receiver.try_recv() {
                self.breakpoints = breakpoint;
            }
            if let Ok(command) = self.control_command_receiver.try_recv() {
                let hook = |core_gui_wrapper: &CoreGuiWrapper| core_gui_wrapper.send_update();
                match command {
                    RunCycle => {
                        self.core.run_cycle(Some(hook), Some(self));
                    }
                    RunInstructions => {
                        self.core.run_instruction(Some(hook), Some(self));
                    }
                    RunUntilAddr => {
                        self.core.run_until_addr(&self.breakpoints, Some(hook), Some(self));
                    }
                    RunEnd => {
                        self.core.run_end(Some(hook), Some(self));
                    }
                    Reset => {
                        self.core.reset();
                        self.send_update();
                    }
                }
                self.control_ack_sender.try_send(()).unwrap();
            }
            if let Ok(byte) = self.console_keyboard_buffer_receiver.try_recv() {
                self.core.keyboard_mmio_ctl.lock().unwrap().append_to_buffer(byte);
            }
            if let Ok(data) = self.load_elf_receiver.try_recv() {
                self.core.load_elf(data.as_slice());
                self.send_update();
            }
        }
    }
}

pub enum ControlCommand {
    RunCycle,
    RunInstructions,
    RunUntilAddr,
    RunEnd,
    Reset,
}

#[derive(Clone)]
pub struct RegisterData {
    pub state: String,
    pub ir: String,
    pub pc: String,
    pub mar: String,
    pub mrdr: String,
    pub mwdr: String,
    pub regfile: [String; 32],
}