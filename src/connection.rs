//! Represents a point-to-point connection, consisting of 2 Channels

use std::{cell::RefCell, rc::Rc};

use crate::channel::Channel;

pub(crate) struct Connection;

impl Connection {
    /// Builds a connection between two channels
    pub fn new() -> (Rc<RefCell<Channel>>, Rc<RefCell<Channel>>) {
        let left = Rc::new(RefCell::new(Channel::new()));
        let right = Rc::new(RefCell::new(Channel::new()));

        left.borrow_mut().add_buddy(right.clone());
        right.borrow_mut().add_buddy(left.clone());

        (left, right)
    }
}