pub mod channel;
pub mod machine;
pub mod node;
pub mod token;
pub mod connection;


use log::trace;
use env_logger::Logger;
use core::time;
use std::cell::RefCell;
use std::sync::Arc;
use std::thread;
use std::{collections::VecDeque, rc::Rc};

use crate::channel::Channel;
use crate::machine::MyCoolMachine;
use crate::node::Node;
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
    let mut client_sockets = SocketSet::new(vec![]);
    let mut server_sockets = SocketSet::new(vec![]);

    let client_socket = tcp::Socket::new(
        tcp::SocketBuffer::new(vec![0; 1500]),
        tcp::SocketBuffer::new(vec![0; 1500]),
    );

    let server_socket = tcp::Socket::new(
        tcp::SocketBuffer::new(vec![0; 1500]),
        tcp::SocketBuffer::new(vec![0; 1500]),
    );

    let client_handle = client_sockets.add(client_socket);
    let server_handle = server_sockets.add(server_socket);

    let remote_addr = IpAddress::v4(1, 2, 3, 5);
    let remote_port = 1234;
    let host_port = 65000;

    loop {
        trace!("=== POLLING CLIENT ===");
        node1.poll(&mut client_sockets);
        trace!("=== POLLING SERVER ===");
        node2.poll(&mut server_sockets);

        let socket = client_sockets.get_mut::<tcp::Socket>(client_handle);
        println!("=== CLIENT SIDE ===");
        println!("State {}", socket.state());
        println!("Active: {}", socket.is_active());
        println!("Listen: {}", socket.is_listening());
        println!("Open: {}", socket.is_open());

        let socket = server_sockets.get_mut::<tcp::Socket>(server_handle);
        println!("=== SERVER SIDE ===");
        println!("State {}", socket.state());
        println!("Active: {}", socket.is_active());
        println!("Listen: {}", socket.is_listening());
        println!("Open: {}", socket.is_open());

        if !socket.is_active() && !socket.is_listening() {
            socket.listen(remote_port).unwrap();
            println!("Server listening...");
        }

        let socket = client_sockets.get_mut::<tcp::Socket>(client_handle);
        if !socket.is_open() {
            socket
                .connect(node1.context(), (remote_addr, remote_port), host_port)
                .unwrap();
            println!("Client connecting...")
        }


        // if socket.can_send() {
        //     println!("sending");
        //     socket.send_slice(b"0123456789abcdef").unwrap();
        //     socket.close();
        // }

        // let socket = sockets.get_mut::<tcp::Socket>(server_handle);
        // if socket.can_recv() {
        //     print!(
        //         "got {:?}",
        //         socket.recv(|buffer| { (buffer.len(), std::str::from_utf8(buffer).unwrap()) })
        //     );
        //     socket.close();
        // }

        // match iface.poll_at(Instant::now()) {
        //     Some(Instant::ZERO) => continue,
        //     Some(d) => println!("sleeping for {}", d),
        //     None => (),
        // }
        thread::sleep(time::Duration::from_secs(1));
    }
}
