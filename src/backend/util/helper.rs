use crate::backend::util::types::Word;

pub fn sign_extend(val: u32, upper_idx: u8) -> Word {
    let msb = (val >> upper_idx) & 0x1;
    if msb == 0x1 {
        let mut tmp = val;
        for i in upper_idx + 1..32 {
            tmp |= 0x1 << i;
        }
        Word::from(tmp)
    } else {
        Word::from(val & (0xFFFFFFFF >> (32 - (upper_idx + 1))))
    }
}

#[macro_export]
macro_rules! send_word {
    ($self:ident, $output:expr, $word:expr) => {
        let out_event = WordEvent::new(
            $self.sim_manager.get_curr_cycle(),
            $word,
            $self.sim_manager.request_new_event_id(),
        );
        send!($self, $output, out_event);
    };
}

#[macro_export]
macro_rules! send_byte {
    ($self:ident, $output:expr, $byte:expr) => {
        let out_event = ByteEvent::new(
            $self.sim_manager.get_curr_cycle(),
            $byte,
            $self.sim_manager.request_new_event_id(),
        );
        send!($self, $output, out_event);
    };
}
