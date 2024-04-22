use std::{io::Write, net::TcpStream};

use crate::{FtpResponseCode, FtpState};

pub fn syst(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    stream
        .write_all(
            FtpResponseCode::SystemName
                .to_string("UNIX Type: L8")
                .as_bytes(),
        )
        .ok()?;
    Some(())
}
