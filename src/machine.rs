use crate::token::{MyRxToken, MyTxToken};
use smoltcp::{
    iface::{Config, Context, Interface, SocketSet},
    phy::{Device, DeviceCapabilities, Medium},
    time::Instant,
    wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address},
};
use std::{borrow::BorrowMut, collections::VecDeque, net::Ipv4Addr};

pub struct MyCoolMachine {
    addr: EthernetAddress,
    buff: VecDeque<Vec<u8>>,
}

impl MyCoolMachine {
    pub fn new(addr: EthernetAddress) -> Self {
        Self {
            addr,
            buff: VecDeque::new(),
        }
    }
}

impl Device for MyCoolMachine {
    type RxToken<'a> = MyRxToken
    where
        Self: 'a;

    type TxToken<'a> = MyTxToken<'a>
    where
        Self: 'a;
    fn receive(&mut self, timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        println!("Cool machine recieving");
        self.buff.pop_front().map(move |buffer| {
            // Create receive token from the popped Vec
            let rx = MyRxToken { buffer };
            // Create transmit token from rest of buff
            let tx = MyTxToken {
                queue: &mut self.buff,
            };
            (rx, tx)
        })
    }

    fn transmit(&mut self, timestamp: Instant) -> Option<Self::TxToken<'_>> {
        println!("Cool machine transmitting");
        Some(MyTxToken {
            queue: &mut self.buff,
        })
    }

    fn capabilities(&self) -> smoltcp::phy::DeviceCapabilities {
        let mut cap = DeviceCapabilities::default();
        cap.medium = Medium::Ethernet;
        cap.max_transmission_unit = 65535;
        cap
    }
}
