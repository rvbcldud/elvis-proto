use std::collections::VecDeque;

use smoltcp::phy::{RxToken, TxToken};

pub struct MyRxToken {
    pub(crate) buffer: Vec<u8>,
}

impl RxToken for MyRxToken {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        println!("Consuming RxToken");
        // Calls the passed closure with the enclosed buffer
        f(&mut self.buffer)
    }
}
pub struct MyTxToken<'a> {
    pub(crate) queue: &'a mut VecDeque<Vec<u8>>,
}

impl<'a> TxToken for MyTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        println!("Consuming TxToken");
        // Create new buffer of specified length
        let mut buffer = vec![0; len];
        // Build packet using passed closure
        let result = f(&mut buffer);
        // Add buffer to back of queue
        self.queue.push_back(buffer);
        // Return ??
        result
    }
}
