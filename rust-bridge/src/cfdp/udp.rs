//! UDP sender for raw CFDP PDUs (dedicated port, bypasses CI_LAB).

use std::io;
use std::net::UdpSocket;

/// Sends raw CFDP PDU bytes (one PDU per UDP datagram).
pub struct CfdpUdpSender {
    socket: UdpSocket,
}

impl CfdpUdpSender {
    /// Binds an ephemeral local port and connects to `addr` (e.g. `127.0.0.1:5235`).
    pub fn connect(addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(addr)?;
        Ok(Self { socket })
    }

    pub fn send_pdu(&self, pdu: &[u8]) -> io::Result<usize> {
        self.socket.send(pdu)
    }
}
