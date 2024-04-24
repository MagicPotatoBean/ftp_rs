use std::net::TcpStream;

use crate::ftp::{FtpState, handshake};

pub fn rein(stream: &mut TcpStream, state: &mut FtpState, _request: Option<String>) -> Option<()> {
    *state = FtpState::default();
    handshake::handshake(stream);
    Some(())
}
