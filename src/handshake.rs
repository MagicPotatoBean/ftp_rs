use std::{io::Write, net::TcpStream};

use crate::FtpResponseCode;

pub fn handshake(stream: &mut TcpStream) -> Option<()> {
    stream
        .write_all(
            FtpResponseCode::CmdOk
                .to_string("Welcome to my shitty FTP server")
                .as_bytes(),
        )
        .ok()
}
