use std::net::TcpStream;

use crate::{FtpCode, FtpState};

pub fn help(stream: &mut TcpStream, _state: &mut FtpState, _request: Option<String>) -> Option<()> {
    FtpCode::HelpMsg.send(stream, "This is a simple, bad FTP server written in rust.").ok()
}
