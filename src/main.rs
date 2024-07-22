pub mod channel;
pub mod machine;
pub mod node;
pub mod token;
pub mod connection;


use log::trace;
use env_logger::Logger;
use smoltcp::socket::tcp::State;
use core::time;
use std::cell::RefCell;
use std::sync::Arc;
use std::thread;
use std::{collections::VecDeque, rc::Rc};

use crate::channel::Channel;
use crate::machine::MyCoolMachine;
use crate::node::{Node, HttpClient, HttpServer};
use crate::connection::Connection;
use smoltcp::{
    iface::{self, Config, Interface, SocketSet},
    phy::{
        Device, DeviceCapabilities, Loopback, Medium, PacketMeta, RxToken, TunTapInterface, TxToken,
    },
    socket::{
        tcp,
        udp::{self, PacketMetadata, UdpMetadata},
    },
    time::{Duration, Instant},
    wire::{EthernetAddress, IpAddress, IpCidr, IpEndpoint, IpListenEndpoint, Ipv6Address},
};

fn main() {
    // Initialize logger for tracing
    env_logger::init();

    // Initialize connection of channels between machines
    let (left, right) = Connection::new();


    // Initialize client machine
    let addr1 = EthernetAddress([0x02, 0x0, 0x0, 0x0, 0x0, 0x01]);
    let machine1 = MyCoolMachine::new(addr1, left.clone());
    let mut node1 = Node::new(machine1);
    node1.add_iface(Config::new(addr1.into()), Instant::now());
    node1.update_ip_addr(IpAddress::v4(1, 2, 3, 4), 24);
    node1.add_ipv4_route([1, 2, 3, 100]);

    // Initialize server machine
    let addr2 = EthernetAddress([0x02, 0x0, 0x0, 0x0, 0x0, 0x02]);
    let machine2 = MyCoolMachine::new(addr2, right.clone());
    let mut node2 = Node::new(machine2);
    node2.add_iface(Config::new(addr2.into()), Instant::now());
    node2.update_ip_addr(IpAddress::v4(1, 2, 3, 5), 24);
    node2.add_ipv4_route([1, 2, 3, 100]);

    // Initialize sockets & socket buffers
    let client_handle = node1.add_tcp_socket();
    let server_handle = node2.add_tcp_socket();

    let remote_addr = IpAddress::v4(1, 2, 3, 5);
    let remote_port = 1234;
    let host_port = 65000;


    loop {
        trace!("=== POLLING CLIENT ===");
        node1.poll(Instant::now());
        trace!("=== POLLING SERVER ===");
        node2.poll(Instant::now());

        println!("=== CLIENT SIDE ===");
        let client_state = node1.socket_status(client_handle);

        println!("=== SERVER SIDE ===");
        let server_state = node2.socket_status(server_handle);

        node2.start_htpp_server(server_handle, remote_port);

        node1.start_http_client(client_handle, remote_addr, remote_port, host_port);

        if client_state == State::Established && server_state == State::Established {
            node1.send_request(client_handle, "GET", "/elvis.html");
            node2.handle_http_server(server_handle);
        }

        thread::sleep(time::Duration::from_secs(1));
    }
}
