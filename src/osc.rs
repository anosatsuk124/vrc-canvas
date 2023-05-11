use std::{
    net::{SocketAddrV4, UdpSocket},
    str::FromStr,
};

use anyhow::Result;
use rosc::OscPacket;

pub const DEFAULT_IP_ADDR: &str = "127.0.0.1";
pub const DEFAULT_RECEIVER_OSC_PORT: usize = 9000;
pub const DEFAULT_SENDER_OSC_PORT: usize = 9001;

pub struct PenHandler {
    pub grabbed: bool,
    position: (f32, f32),
    triggered: bool,
}

impl Default for PenHandler {
    fn default() -> Self {
        Self {
            grabbed: false,
            position: (0.0, 0.0),
            triggered: false,
        }
    }
}

pub fn send_packet(addr: &str) -> Result<()> {
    let receiver_addr = std::net::SocketAddrV4::from_str(
        format!("{}:{}", DEFAULT_IP_ADDR, DEFAULT_RECEIVER_OSC_PORT).as_str(),
    )?;

    let sender_addr = std::net::SocketAddrV4::from_str(
        format!("{}:{}", DEFAULT_IP_ADDR, DEFAULT_SENDER_OSC_PORT).as_str(),
    )?;

    let socket = UdpSocket::bind(sender_addr)?;

    let packet = rosc::OscPacket::Message(rosc::OscMessage {
        addr: addr.to_string(),
        args: vec![rosc::OscType::Int(1)],
    });

    let encoded_data = rosc::encoder::encode(&packet)?;

    socket.send_to(encoded_data.as_slice(), receiver_addr)?;

    Ok(())
}

fn handle_packet(packet: OscPacket) {
    match packet {
        OscPacket::Message(msg) => {
            println!("message: {} {:?}", msg.addr, msg.args);
        }
        OscPacket::Bundle(bundle) => {
            println!("bundle: {:?}", bundle);
        }
    }
}
