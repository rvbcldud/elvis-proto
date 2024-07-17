//! This defines a channel of communication between two nodes

use std::{cell::RefCell, rc::Rc};
use log::trace;

pub(crate) struct Channel {
    buffer: Vec<u8>,
    buddy: Option<Rc<RefCell<Channel>>>,
}

impl Channel {
    /// Constructs a new channel with an empty buffer vec & no buddy :(
    pub fn new() -> Self {
        Self { 
            buffer: Vec::new(),
            buddy: None,
        }
    }

    /// Adds a "buddy" -> an opposite side to the connection to write to
    pub fn add_buddy(&mut self, buddy: Rc<RefCell<Channel>>) {
        self.buddy = Some(buddy);
    }

    /// Writes to other side of connection
    pub fn write(&mut self, packet: Vec<u8>) {
        trace!("-- CHANNEL WRITE --");
        self.buddy.as_mut().unwrap().borrow_mut().buffer = packet;
        self.buddy.as_mut().unwrap().borrow_mut().contents();
    }

    /// Reads from correct side of connection
    pub fn read(&mut self) -> Vec<u8> {
        trace!("-- CHANNEL READ --");
        self.contents();
        let result = self.buffer.clone();
        self.buffer.clear();

        result
    }

    /// Checks if the channel buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Traces the connents of the channel buffer
    pub fn contents(&self) {
        trace!("CHANNEL CONTENTS -> {:?}", self.buffer);
    }
}
