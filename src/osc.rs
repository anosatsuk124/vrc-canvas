pub mod pen_handle;

use std::{
    net::{SocketAddrV4, UdpSocket},
    str::FromStr,
};

use anyhow::Result;
use rosc::OscPacket;

pub const DEFAULT_BASE_ADDR: &str = "/avatar/parameters";
pub const DEFAULT_ADDR: &str = "";

pub const DEFAULT_IP_ADDR: &str = "127.0.0.1";
pub const DEFAULT_RECEIVER_OSC_PORT: usize = 9000;
pub const DEFAULT_SENDER_OSC_PORT: usize = 9001;

#[derive(Debug)]
pub struct OscHandler {
    pub socket: UdpSocket,
}

pub static OSC_HANDLER: once_cell::sync::OnceCell<OscHandler> = once_cell::sync::OnceCell::new();

pub const CANVAS_UPDATE_LATENCY_DEFAULT: std::time::Duration =
    std::time::Duration::from_millis(100);

impl OscHandler {
    pub fn init_hadler() -> Result<()> {
        let sender_addr = SocketAddrV4::from_str(
            format!("{}:{}", DEFAULT_IP_ADDR, DEFAULT_SENDER_OSC_PORT).as_str(),
        )?;

        let handler = OscHandler {
            socket: UdpSocket::bind(sender_addr)?,
        };

        if let Err(e) = OSC_HANDLER.set(handler) {
            anyhow::bail!("failed to init osc handler: {:?}", e);
        } else {
            log::info!("Started recieving from {}", sender_addr);
        }

        Ok(())
    }

    pub fn get_handler() -> Result<&'static OscHandler> {
        OSC_HANDLER
            .get()
            .ok_or(anyhow::anyhow!("OSC Handler is not initialized"))
    }
}

pub fn start_osc(current_state: Option<pen_handle::PenState>) -> Result<()> {
    OscHandler::init_hadler()?;

    pen_handle::PenHandler::init(None)?;

    tokio::spawn(async {
        loop {
            let handler = pen_handle::PEN_HANDLER.get().unwrap().lock().await;
            handler.eval().await;
            tokio::time::sleep(CANVAS_UPDATE_LATENCY_DEFAULT).await;
        }
    });

    Ok(())
}

pub fn receive_packet(buf: &mut [u8]) -> Result<OscPacket> {
    let handler = OscHandler::get_handler()?;

    let socket = &handler.socket;

    match socket.recv_from(buf) {
        Ok((size, addr)) => {
            let (_buf, packet) = rosc::decoder::decode_udp(&buf[..size])?;
            log::info!("Received {:?} from {}", packet, addr);

            return Ok(packet);
        }
        Err(e) => {
            log::error!("Error receiving from socket: {}", e);
            return Err(anyhow::anyhow!("Error receiving from socket: {}", e));
        }
    }
}

/*
pub async fn expect_packet(addr: &str, value: Vec<OscType>) -> Result<OscPacket> {
    let mut osc_buffer = [0u8; rosc::decoder::MTU];

    loop {
        let packet = receive_packet(&mut osc_buffer)?;
        handle_packet(&packet);
        if let OscPacket::Message(msg) = packet.clone() {
            if msg.addr == addr && msg.args == value {
                return Ok(packet);
            }
        }
    }
}
*/

fn handle_packet(packet: &OscPacket) {
    match packet {
        OscPacket::Message(msg) => {
            log::info!("message: {} {:?}", msg.addr, msg.args);
        }
        OscPacket::Bundle(bundle) => {
            log::info!("bundle: {:?}", bundle);
        }
    }
}

pub fn send_packet(addr: &str, value: Vec<rosc::OscType>) -> Result<()> {
    let socket = &OscHandler::get_handler()?.socket;

    let receiver_addr = SocketAddrV4::from_str(
        format!("{}:{}", DEFAULT_IP_ADDR, DEFAULT_RECEIVER_OSC_PORT).as_str(),
    )?;

    let addr = format!("{}{}{}", DEFAULT_BASE_ADDR, DEFAULT_ADDR, addr);

    let packet = rosc::OscPacket::Message(rosc::OscMessage {
        addr: addr.to_string(),
        args: value,
    });

    let encoded_data = rosc::encoder::encode(&packet)?;

    socket.send_to(encoded_data.as_slice(), receiver_addr)?;
    log::info!("Sending {:?} to {}/{}", &packet, &receiver_addr, &addr);

    Ok(())
}
