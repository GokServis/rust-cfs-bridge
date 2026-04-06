//! Sends a sample [`rust_bridge::CcsdsPacket`] to `127.0.0.1:1234` using the library serializer.

use rust_bridge::{CcsdsPacket, SpaceCommand, UdpSender};

fn main() -> std::io::Result<()> {
    let cmd = SpaceCommand::from_json(r#"{"command":"CMD_HEARTBEAT","sequence_count":0}"#)
        .expect("valid dictionary command");
    let packet = CcsdsPacket::from_command(&cmd).expect("valid command");

    let sender = UdpSender::connect("127.0.0.1:1234")?;
    let n = sender.send_packet(&packet)?;
    println!("rust-bridge: sent {n} bytes to 127.0.0.1:1234 (CCSDS + CRC wire format)");
    // Allow CI_LAB → SB → bridge_reader time to run before the container exits (Docker foreground PID).
    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(())
}
