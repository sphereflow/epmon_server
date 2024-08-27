use local_ip_address::local_ip;
use std::{net::UdpSocket, sync::*, thread, time::Duration};

pub fn udp_broadcast(connected: Arc<Mutex<bool>>) {
    let recv_sock = UdpSocket::bind("0.0.0.0:9900").expect("could not bind recv_sock");
    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
        let mut bconnected;
        let last_addr_byte = match local_ip().expect("getting local ip failed") {
            std::net::IpAddr::V4(ip) => ip.octets()[3],
            std::net::IpAddr::V6(_) => panic!("expected ipv4 address but got ipv6"),
        };
        println!("last_addr_byte : {last_addr_byte}");
        sock.set_broadcast(true)
            .expect("could not set broadcast to true");
        loop {
            // check if connected
            if let Ok(mgc) = connected.lock() {
                bconnected = *mgc;
            } else {
                println!("udp_broadcast: Could not lock mutex");
                break;
            }

            // no connection => send broadcast packet
            if !bconnected {
                sock.send_to(&[last_addr_byte], "192.168.178.255:9900")
                    .expect("can not send last_addr_byte");
                println!("packet sent");
                let mut buf = [0];
                recv_sock.recv(&mut buf).expect("recv failed");
                println!("received: {buf:?}");
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}
