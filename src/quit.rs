use std::{io::Write, net::TcpStream};

use crate::{FtpCode, FtpState};

pub fn quit(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    stream.shutdown(std::net::Shutdown::Both).ok()?;
    None
}
