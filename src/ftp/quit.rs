use std::net::TcpStream;

use crate::ftp::FtpState;

pub fn quit(stream: &mut TcpStream, _state: &mut FtpState, _request: Option<String>) -> Option<()> {
    stream.shutdown(std::net::Shutdown::Both).ok()?;
    None
}
