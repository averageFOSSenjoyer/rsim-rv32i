use crate::backend::util::types::{Byte, Word};
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
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt::{Debug, Formatter};
use std::ops::Range;
use std::sync::{Arc, Mutex};
use elf::ElfBytes;
use elf::endian::LittleEndian;

#[ComponentAttribute({
"port": {
    "input": [
        ["cpu_addr", "Word"],

        ["cpu_wdata", "Word"],
        ["cpu_read_en", "Byte"],
        ["cpu_rmask", "Byte"],
        ["cpu_write_en", "Byte"],
        ["cpu_wmask", "Byte"]
    ],
    "output": [
        ["cpu_rdata", "Word"],
        ["cpu_resp", "Byte"]
    ],
    "clock": true
}
})]
#[allow(dead_code)]
pub struct MemCtl {
    pub backend_mem: BTreeMap<Word, Byte>,
    pub label: BTreeMap<Word, String>,
    mmio_ctl: HashMap<Range<u32>, Arc<Mutex<dyn MmioCtl>>>,
    is_busy: bool,
}

impl MemCtl {
    pub fn new(
        component_id: ComponentId,
        sim_manager: Arc<SimManager>,
        ack_sender: Sender<EventId>,
        cpu_addr: Rx<Word>,
        cpu_wdata: Rx<Word>,
        cpu_read_en: Rx<Byte>,
        cpu_rmask: Rx<Byte>,
        cpu_write_en: Rx<Byte>,
        cpu_wmask: Rx<Byte>,
        cpu_rdata: Tx<Word>,
        cpu_resp: Tx<Byte>,
    ) -> Self {
        let clock_channel = unbounded();

        MemCtl {
            backend_mem: Default::default(),
            label: Default::default(),
            mmio_ctl: Default::default(),
            is_busy: false,
            component_id,
            sim_manager,
            ack_sender,
            clock_sender: clock_channel.0,
            clock_receiver: clock_channel.1,
            cpu_addr,
            cpu_wdata,
            cpu_read_en,
            cpu_rmask,
            cpu_write_en,
            cpu_wmask,
            cpu_rdata,
            cpu_resp,
        }
    }

    fn init_impl(&mut self) {}

    fn reset_impl(&mut self) {
        self.backend_mem.clear();
        self.label.clear();
        self.mmio_ctl.clear();
    }

    fn poll_impl(&mut self) {}

    pub fn install_mmio_ctl(&mut self, addr_range: Range<u32>, mmio_ctl: Arc<Mutex<dyn MmioCtl>>) {
        self.mmio_ctl.insert(addr_range, mmio_ctl);
    }

    fn on_clock(&mut self) {
        //can recv request
        if !self.is_busy {
            // a r/w request came in
            if self.cpu_write_en.get_value().is_something_nonzero() {
                if let Some(wmask) = Into::<Option<u8>>::into(self.cpu_wmask.get_value()) {
                    for i in 0..4 {
                        if wmask >> i & 0x1 == 0x1 {
                            let addr_idx = (self.cpu_addr.get_value() & Word::from(0xFFFFFFFCu32))
                                + Word::from(i as u32);
                            let data = self.cpu_wdata.get_value()[i]
                                .map(Byte::from)
                                .unwrap_or(Byte::unknown());

                            let mut written_to_mmio = false;
                            for (addr_range, mmio_ctl) in self.mmio_ctl.iter_mut() {
                                if let Some(addr_idx_u32) = addr_idx.into() {
                                    if addr_range.contains(&addr_idx_u32) {
                                        mmio_ctl.lock().unwrap().write(addr_idx, data);
                                        written_to_mmio = true;
                                    }
                                }
                            }

                            if !written_to_mmio {
                                self.backend_mem.insert(addr_idx, data);
                            }
                        }
                    }
                }
                self.cpu_resp.send(Byte::from(1u8), 0);
            } else if self.cpu_read_en.get_value().is_something_nonzero() {
                let mut ret = Word::unknown();
                if let Some(rmask) = Into::<Option<u8>>::into(self.cpu_rmask.get_value()) {
                    for i in 0..4 {
                        if rmask >> i & 0x1 == 0x1 {
                            let addr_idx = (self.cpu_addr.get_value() & Word::from(0xFFFFFFFCu32))
                                + Word::from(i as u32);
                            if self.backend_mem.contains_key(&addr_idx) {
                                ret[i] = self.backend_mem[&addr_idx].into()
                            } else {
                                for (addr_range, mmio_ctl) in self.mmio_ctl.iter_mut() {
                                    if let Some(addr_idx_u32) = addr_idx.into() {
                                        if addr_range.contains(&addr_idx_u32) {
                                            ret[i] = mmio_ctl.lock().unwrap().read(addr_idx).into();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                self.cpu_rdata.send(ret, 0);
                self.cpu_resp.send(Byte::from(1u8), 0);
            } else {
                self.cpu_resp.send(Byte::from(0u8), 0);
            }
        }
    }

    fn on_comb(&mut self) {}

    pub fn load_elf(&mut self, data: &[u8]) {
        ElfBytes::<LittleEndian>::minimal_parse(data).map(|elf_bytes| {
            // symbol table
            if let Ok(Some((symbol_table, string_table))) = elf_bytes.symbol_table() {
                for symbol in symbol_table.iter() {
                    if symbol.st_name != 0 {
                        if let Ok(symbol_name) = string_table.get(symbol.st_name as usize) {
                            self.label.insert(Word::from(symbol.st_value as u32), symbol_name.to_string());
                        }
                    }
                }
            }
            // sections
            if let Some(section_table) = elf_bytes.section_headers() {
                for section_header in section_table.iter() {
                    if let Ok((section_data, _)) = elf_bytes.section_data(&section_header) {
                        let addr = Word::from(section_header.sh_addr as u32);
                        for i in 0..section_data.len() as u32 {
                            self.backend_mem
                                .insert(addr + Word::from(i), section_data[i as usize].into());
                        }
                    }
                }
            }
        }).unwrap_or_else(|_| println!("Failed to parse ELF file"));
    }
}

impl Debug for MemCtl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MemCtl: {{cpu_addr: {:?}, wmask: {:?}}}",
            self.cpu_addr.get_value(),
            self.cpu_wmask.get_value()
        )
    }
}

pub trait MmioCtl: Send + Sync + 'static {
    fn read(&mut self, addr: Word) -> Byte;
    fn write(&mut self, addr: Word, data: Byte);
    fn reset(&mut self);
}

pub struct KeyboardMmioCtl {
    buffer: VecDeque<u8>,
}

impl KeyboardMmioCtl {
    pub const STATUS_ADDR: u32 = 0x000A0000;
    pub const DATA_ADDR: u32 = 0x000A0001;

    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
        }
    }

    pub fn append_to_buffer(&mut self, data: u8) {
        self.buffer.push_back(data);
    }
}

impl MmioCtl for KeyboardMmioCtl {
    fn read(&mut self, addr: Word) -> Byte {
        if let Some(addr) = Into::<Option<u32>>::into(addr) {
            if addr == Self::STATUS_ADDR {
                if !self.buffer.is_empty() {
                    Byte::from(1u8)
                } else {
                    Byte::from(0u8)
                }
            } else if addr == Self::DATA_ADDR {
                if !self.buffer.is_empty() {
                    Byte::from(self.buffer.pop_front().unwrap())
                } else {
                    Byte::unknown()
                }
            } else {
                Byte::unknown()
            }
        } else {
            Byte::unknown()
        }
    }

    fn write(&mut self, _addr: Word, _data: Byte) {
        // absolutely nothing happens
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }
}

const NUM_ROWS: usize = 25;
const NUM_COLS: usize = 80;

pub struct VgaMmioCtl {
    buffer: [u8; NUM_ROWS * NUM_COLS * 2],
}

impl VgaMmioCtl {
    pub const NUM_ROWS: usize = NUM_ROWS;
    pub const NUM_COLS: usize = NUM_COLS;
    pub const NUM_BYTES: usize = Self::NUM_ROWS * Self::NUM_COLS * 2;
    pub const BASE_ADDR: u32 = 0x000B_8000;

    pub fn new() -> Self {
        let mut buffer = [0u8; Self::NUM_BYTES];
        for (i, buf) in buffer.iter_mut().enumerate().take(Self::NUM_BYTES) {
            if i % 2 == 0 {
                *buf = 0u8;
            } else {
                *buf = 0x0F; // black bg white fg
            }
        }
        Self { buffer }
    }

    pub fn get_buffer(&mut self) -> &[u8; Self::NUM_BYTES] {
        &self.buffer
    }
}

impl MmioCtl for VgaMmioCtl {
    fn read(&mut self, addr: Word) -> Byte {
        if let Some(addr) = Into::<Option<u32>>::into(addr) {
            let buffer_idx = addr - Self::BASE_ADDR;
            if buffer_idx <= Self::NUM_BYTES as u32 {
                Byte::from(self.buffer[buffer_idx as usize])
            } else {
                Byte::unknown()
            }
        } else {
            Byte::unknown()
        }
    }

    fn write(&mut self, addr: Word, data: Byte) {
        if let Some(addr) = Into::<Option<u32>>::into(addr) {
            let buffer_idx = addr - Self::BASE_ADDR;
            if buffer_idx <= (Self::NUM_ROWS * Self::NUM_COLS * 2) as u32 {
                if let Some(data) = Into::<Option<u8>>::into(data) {
                    self.buffer[buffer_idx as usize] = data;
                }
            }
        }
    }

    fn reset(&mut self) {
        let mut buffer = [0u8; Self::NUM_BYTES];
        for (i, buf) in buffer.iter_mut().enumerate().take(Self::NUM_BYTES) {
            if i % 2 == 0 {
                *buf = 0u8;
            } else {
                *buf = 0x0F; // black bg white fg
            }
        }
        self.buffer = buffer;
    }
}
