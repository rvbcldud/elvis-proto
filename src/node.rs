use smoltcp::{
    iface::{Config, Context, Interface, SocketSet},
    phy::Device,
    time::Instant,
    wire::{IpAddress, IpCidr},
};

/// This is a wrapper around Interface and a Device
pub struct Node<D> {
    iface: Option<Interface>,
    device: D,
}

impl<D> Node<D>
where
    D: Device,
{
    /// Initialize Node with device, with no interface to begin with
    pub fn new(device: D) -> Self {
        Self {
            iface: None,
            device,
        }
    }

    /// Add interface given configuration and timestamp
    pub fn add_iface(&mut self, config: Config, timestamp: Instant) {
        self.iface = Some(Interface::new(config, &mut self.device, timestamp));
    }

    /// Updates the ip address of the interface
    pub fn update_ip_addr(&mut self, addr: IpAddress, subnet: u8) {
        self.iface.as_mut().unwrap().update_ip_addrs(|addrs| {
            addrs.push(IpCidr::new(addr, subnet)).unwrap();
        });
    }

    /// Transmit packets queued in the given sockets, and receive packets queued in the device
    pub fn poll(&mut self, sockets: &mut SocketSet) {
        self.iface
            .as_mut()
            .unwrap()
            .poll(Instant::now(), &mut self.device, sockets);
    }

    /// Gets the context of the Interface (useful for socket calls)
    pub fn context(&mut self) -> &mut Context {
        self.iface.as_mut().unwrap().context()
    }
}
