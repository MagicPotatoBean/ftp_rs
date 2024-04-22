use std::{io::Write, net::TcpStream};

use crate::{FtpResponseCode, FtpState};

pub fn help(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    stream
        .write_all(
            FtpResponseCode::HelpMsg
                .to_string("This is a simple, bad FTP server written in rust.")
                .as_bytes(),
        )
        .ok()
}
