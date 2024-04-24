use std::net::TcpStream;

use crate::ftp::{FtpCode, FtpState};

pub fn syst(stream: &mut TcpStream, _state: &mut FtpState, _request: Option<String>) -> Option<()> {
    FtpCode::SystemName.send(stream, "UNIX Type: L8").ok()?;
    Some(())
}
