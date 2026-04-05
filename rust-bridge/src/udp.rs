//! UDP sender for CCSDS bridge wire format (primary header + payload + CRC-16 trailer).

use std::io;
use std::net::UdpSocket;

use crate::CcsdsPacket;

/// Sends [`CcsdsPacket`] buffers built by [`CcsdsPacket::to_bytes`].
pub struct UdpSender {
    socket: UdpSocket,
}

impl UdpSender {
    /// Binds an ephemeral local port and connects to `addr` (e.g. `127.0.0.1:1234`).
    pub fn connect(addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(addr)?;
        Ok(Self { socket })
    }

    /// Serializes the packet and sends the full datagram (including CRC trailer).
    pub fn send_packet(&self, packet: &CcsdsPacket) -> io::Result<usize> {
        self.socket.send(&packet.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use std::net::UdpSocket;
    use std::thread;
    use std::time::Duration;

    use super::*;
    use crate::{CcsdsPacket, SpaceCommand};

    #[test]
    fn connect_and_send_packet_delivers_bytes() {
        let recv = UdpSocket::bind("127.0.0.1:0").expect("bind receiver");
        recv.set_read_timeout(Some(Duration::from_secs(2)))
            .expect("set_read_timeout");
        let addr = recv.local_addr().expect("local_addr");

        let handle = thread::spawn(move || {
            let mut buf = [0u8; 2048];
            let (n, _) = recv.recv_from(&mut buf).expect("recv_from");
            buf[..n].to_vec()
        });

        thread::sleep(Duration::from_millis(20));

        let cmd = SpaceCommand {
            apid: 0x10,
            sequence_count: 3,
            payload: vec![0x01, 0x02],
        };
        let packet = CcsdsPacket::from_command(&cmd).expect("packet");
        let expected = packet.to_bytes();

        let sender = UdpSender::connect(&addr.to_string()).expect("connect");
        let sent = sender.send_packet(&packet).expect("send_packet");
        assert_eq!(sent, expected.len());

        let got = handle.join().expect("thread");
        assert_eq!(got, expected);
    }
}
