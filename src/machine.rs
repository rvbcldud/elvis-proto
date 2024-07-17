use crate::{
    channel::Channel,
    token::{MyRxToken, MyTxToken},
};
use smoltcp::{
    iface::{Config, Context, Interface, SocketSet},
    phy::{Device, DeviceCapabilities, Medium},
    time::Instant,
    wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address},
};
use std::{cell::RefCell, collections::VecDeque, net::Ipv4Addr, rc::Rc};
use log::trace;

pub struct MyCoolMachine {
    addr: EthernetAddress,
    channel: Rc<RefCell<Channel>>,
}

impl MyCoolMachine {
    pub fn new(addr: EthernetAddress, channel: Rc<RefCell<Channel>>) -> Self {
        Self {
            addr,
            channel
        }
    }
}

impl Device for MyCoolMachine {
    type RxToken<'a> = MyRxToken
    where
        Self: 'a;

    type TxToken<'a> = MyTxToken
    where
        Self: 'a;
    fn receive(&mut self, timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        trace!("Machine recieving");
        let rx = MyRxToken {
            channel: self.channel.clone(),
        };
        let tx = MyTxToken {
            channel: self.channel.clone(),
        };

        let c = self.channel.borrow_mut();
        // Only do something when the channel is full
        if !c.is_empty() {
            trace!("-- CHANNEL IS [NOT] EMPTY --");
            Some((rx, tx))
        } else {
            trace!("-- CHANNEL IS EMPTY --");
            None
        }
    }

    fn transmit(&mut self, timestamp: Instant) -> Option<Self::TxToken<'_>> {
        trace!("Machine transmitting");
        Some(MyTxToken {
            channel: self.channel.clone(),
        })
    }

    fn capabilities(&self) -> smoltcp::phy::DeviceCapabilities {
        let mut cap = DeviceCapabilities::default();
        cap.medium = Medium::Ethernet;
        cap.max_transmission_unit = 65535;
        cap
    }
}
