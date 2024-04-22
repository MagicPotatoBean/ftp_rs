use std::{io::Write, net::TcpStream};

use crate::{FtpCode, FtpState};

pub fn help(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    FtpCode::HelpMsg.send(stream, "This is a simple, bad FTP server written in rust.").ok()
}
