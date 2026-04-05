//! Sends a sample [`rust_bridge::CcsdsPacket`] to `127.0.0.1:1234` using the library serializer.

use rust_bridge::{CcsdsPacket, SpaceCommand, UdpSender};

fn main() -> std::io::Result<()> {
    let cmd = SpaceCommand {
        apid: 0x7B,
        sequence_count: 0,
        payload: vec![0xC0, 0xFF, 0xEE],
    };
    let packet = CcsdsPacket::from_command(&cmd).expect("valid command");

    let sender = UdpSender::connect("127.0.0.1:1234")?;
    let n = sender.send_packet(&packet)?;
    println!("rust-bridge: sent {n} bytes to 127.0.0.1:1234 (CCSDS + CRC wire format)");
    Ok(())
}
