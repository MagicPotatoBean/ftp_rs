use std::{io::Write, net::TcpStream};

use crate::FtpCode;

pub fn handshake(stream: &mut TcpStream) -> Option<()> {
    FtpCode::CmdOk.send(stream, "Welcome to my RFC959 FTP server").ok()
}
