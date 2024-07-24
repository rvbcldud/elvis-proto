use std::{borrow::BorrowMut, str::FromStr};

use smoltcp::{
    iface::{Config, Context, Interface, SocketHandle, SocketSet}, phy::Device, socket::{dhcpv4::Socket, tcp::{self, State}}, time::Instant, wire::{IpAddress, IpCidr, IpEndpoint, IpListenEndpoint, Ipv4Address}
};

pub trait HttpServer {
    fn start_htpp_server(&mut self, handle: SocketHandle, remote_port: u16);
    fn handle_http_server(&mut self, handle: SocketHandle);
}

pub trait HttpClient {
    fn start_http_client(&mut self, handle: SocketHandle, remote_addr: IpAddress, remote_port: u16, host_port: u16);
    fn send_request(&mut self, handle: SocketHandle, method: &str, path: &str);
    fn handle_http_client(&mut self, handle: SocketHandle);
}

/// This is a wrapper around Interface and a Device
pub struct Node<D> {
    iface: Option<Interface>,
    device: D,
    sockets: SocketSet<'static>
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
            sockets: SocketSet::new(vec![]),
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

    /// Add a default IPv4 gateway
    pub fn add_ipv4_route(&mut self, addr: [u8; 4]) {
        self.iface
            .as_mut()
            .unwrap()
            .routes_mut()
            .add_default_ipv4_route(Ipv4Address::from_bytes(&addr))
            .unwrap();
    }

    /// Transmit packets queued in the given sockets, and receive packets queued in the device
    pub fn poll(&mut self, timestamp: Instant) {
        let _ = self
            .iface
            .as_mut()
            .unwrap()
            .poll(timestamp, &mut self.device, &mut self.sockets);
    }

    pub fn delay(&mut self, sockets: &mut SocketSet) {
        let _ = self
            .iface
            .as_mut()
            .unwrap()
            .poll_delay(Instant::now(), sockets);
    }

    /// Gets the context of the Interface (useful for socket calls)
    pub fn context(&mut self) -> &mut Context {
        self.iface.as_mut().unwrap().context()
    }

    /// Adds a TCP socket to the device
    pub fn add_tcp_socket(&mut self) -> SocketHandle {
        let socket = tcp::Socket::new(
            tcp::SocketBuffer::new(vec![0; 1500]),
            tcp::SocketBuffer::new(vec![0; 1500]),
        );
        
        self.sockets.add(socket)
    }

    /// Prints the status of a socket, given a SocketHandle
    pub fn socket_status(&mut self, handle: SocketHandle) -> State {
        let socket = self.sockets.get_mut::<tcp::Socket>(handle);
        println!("State {}", socket.state());
        println!("Active: {}", socket.is_active());
        println!("Listen: {}", socket.is_listening());
        println!("Open: {}", socket.is_open());

        socket.state()
    }

    /// Connects socket with given handle to the remote endpoint (addr, port)
    pub fn socket_connect(&mut self, handle: SocketHandle, remote_addr: IpAddress, remote_port: u16, host_port: u16) {
        let socket = self.sockets.get_mut::<tcp::Socket>(handle);
        let remote_endpoint: IpEndpoint = (remote_addr, remote_port).into();
        let local_endpoint: IpListenEndpoint = host_port.into();

        if !socket.is_open() {
            println!("Client listening...");
            socket.connect(
                self.iface.as_mut().unwrap().context(),
                remote_endpoint, 
                local_endpoint
            ).unwrap();
        } else {
            println!("Socket is already open!");
        }
    }

    /// Starts a socket listening on a given port
    pub fn socket_listen(&mut self, handle: SocketHandle, remote_port: u16) {
        let socket = self.sockets.get_mut::<tcp::Socket>(handle);

        if !socket.is_active() && !socket.is_listening() {
            println!("Server listening...");
            socket.listen(remote_port).unwrap();
        } else {
            println!("Socket already active and listening!");
        }
    }

    /// Receives a packet from a given socket, if possible
    pub fn socket_recv<'b, F, R>(&'b mut self, handle: SocketHandle, f: F)
        where
            F: FnOnce(&'b mut [u8]) -> (usize, R),
        {
        let socket = self.sockets.get_mut::<tcp::Socket>(handle);

        if socket.can_recv() {
            // Perform closure on received packet
            let _ = socket.recv(f);
            // socket.
        } else {
            println!("Cannot recv!");
        }
    }

    /// Sends a packet to a given socket, if possible
    pub fn socket_send(&mut self, handle: SocketHandle, packet: &[u8]) {
        let socket = self.sockets.get_mut::<tcp::Socket>(handle);

        if socket.can_send() {
            socket.send_slice(packet).unwrap();
        } else {
            println!("Cannot send!");
        }
    }
    
}


impl<MyCoolMachine> HttpServer for Node<MyCoolMachine>
where
    MyCoolMachine: Device,
{
    fn start_htpp_server(&mut self, handle: SocketHandle, remote_port: u16) {
        self.socket_listen(handle, remote_port);
    }

    fn handle_http_server(&mut self, handle: SocketHandle) {
        let packet = {
            let mut packet_data = Vec::new(); // Create a vector to store packet data
            self.socket_recv(handle, |recv_packet| {
                packet_data.extend_from_slice(recv_packet);
                (recv_packet.len(), ())
            });
            packet_data
        };
        println!("Received REQUEST: {}", std::str::from_utf8(&packet).unwrap());

        let response = "HTTP Response!";
        self.socket_send(handle, response.as_bytes());
    }
}

impl<MyCoolMachine> HttpClient for Node<MyCoolMachine>
where
    MyCoolMachine: Device,
{
    fn start_http_client(&mut self, handle: SocketHandle, remote_addr: IpAddress, remote_port: u16, host_port: u16) {
        self.socket_connect(handle, remote_addr, remote_port, host_port);

    }

    fn send_request(&mut self, handle: SocketHandle, method: &str, path: &str) {
        let packet = format!("{} {}", method, path);

        println!("Sending REQUEST: {}", packet);

        self.socket_send(handle, packet.as_bytes());
    }

    fn handle_http_client(&mut self, handle: SocketHandle) {
        self.socket_recv(handle, |packet| {
            println!("Receieved RESPONSE: {}", std::str::from_utf8(packet).unwrap());
            (packet.len(), ())
        })
    }
    
}
