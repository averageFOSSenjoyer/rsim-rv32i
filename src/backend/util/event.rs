use crate::backend::util::types::{Byte, Word};
use rsim_core::event::Event;
use rsim_core::types::{Cycle, EventId};
use std::any::Any;
use std::fmt::Debug;

// todo: generic events?

#[derive(Clone, Debug)]
pub struct WordEvent {
    scheduled_time: Cycle,
    event_id: EventId,
    data: Word,
}

impl WordEvent {
    pub fn new(scheduled_time: Cycle, data: Word, event_id: EventId) -> Self {
        Self {
            scheduled_time,
            event_id,
            data,
        }
    }
}

impl Event for WordEvent {
    fn get_event_id(&self) -> EventId {
        self.event_id
    }

    fn get_scheduled_time(&self) -> Cycle {
        self.scheduled_time
    }

    fn get_data_as_any(&self) -> Box<dyn Any> {
        Box::new(self.data)
    }
}

#[derive(Clone, Debug)]
pub struct ByteEvent {
    scheduled_time: Cycle,
    event_id: EventId,
    data: Byte,
}

impl ByteEvent {
    pub fn new(scheduled_time: Cycle, data: Byte, event_id: EventId) -> Self {
        Self {
            scheduled_time,
            event_id,
            data,
        }
    }
}

impl Event for ByteEvent {
    fn get_event_id(&self) -> EventId {
        self.event_id
    }

    fn get_scheduled_time(&self) -> Cycle {
        self.scheduled_time
    }

    fn get_data_as_any(&self) -> Box<dyn Any> {
        Box::new(self.data)
    }
}
