pub mod machine;
pub mod node;
pub mod token;

use core::time;
use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;

use crate::machine::MyCoolMachine;
use crate::node::Node;
use smoltcp::{
    iface::{self, Config, Interface, SocketSet}, 
    phy::{Device, DeviceCapabilities, Loopback, Medium, PacketMeta, RxToken, TunTapInterface, TxToken}, 
    socket::{tcp, udp::{self, PacketMetadata, UdpMetadata}}, 
    time::{Duration, Instant}, 
    wire::{EthernetAddress, IpAddress, IpCidr, IpEndpoint, IpListenEndpoint, Ipv6Address}};

fn main() {
    let addr = EthernetAddress([0x02, 0x0, 0x0, 0x0, 0x0, 0x01]);

    let mut machine = MyCoolMachine::new(addr);
    let mut node = Node::new(machine);

    node.add_iface(Config::new(addr.into()), Instant::now());

    node.update_ip_addr(IpAddress::v4(1, 2, 3, 4), 8);

    let mut sockets = SocketSet::new(vec![]);

    let client_socket = tcp::Socket::new(
        tcp::SocketBuffer::new(vec![0; 1500]),
        tcp::SocketBuffer::new(vec![0; 1500]),
    );

    let server_socket = tcp::Socket::new(
        tcp::SocketBuffer::new(vec![0; 1500]),
        tcp::SocketBuffer::new(vec![0; 1500]),
    );

    let client_handle = sockets.add(client_socket);
    let server_handle = sockets.add(server_socket);

    let remote_addr = IpAddress::v4(1, 2, 3, 4);
    let remote_port = 1234;
    let host_port = 65000;

    loop {
        node.poll(&mut sockets);

        let socket = sockets.get_mut::<tcp::Socket>(client_handle);
        println!("=== CLIENT SIDE ===");
        println!("State {}", socket.state());
        println!("Active: {}", socket.is_active());
        println!("Listen: {}", socket.is_listening());
        println!("Open: {}", socket.is_open());


        let socket = sockets.get_mut::<tcp::Socket>(server_handle);
        println!("=== SERVER SIDE ===");
        println!("State {}", socket.state());
        println!("Active: {}", socket.is_active());
        println!("Listen: {}", socket.is_listening());
        println!("Open: {}", socket.is_open());

        if !socket.is_active() && !socket.is_listening() {
            socket.listen(remote_port).unwrap();
            println!("Server listening");
        }

        let socket = sockets.get_mut::<tcp::Socket>(client_handle);
        if !socket.is_open() {
            socket
                .connect(node.context(), (remote_addr, remote_port), host_port)
                .unwrap();
        }

        // if socket.can_recv() {
        //     print!(
        //         "got {:?}",
        //         socket.recv(|buffer| { (buffer.len(), std::str::from_utf8(buffer).unwrap()) })
        //     );
        //     socket.close();
        // }

        // if socket.can_send() {
        //     println!("sending");
        //     socket.send_slice(b"0123456789abcdef").unwrap();
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
