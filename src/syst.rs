use std::{io::Write, net::TcpStream};

use crate::{FtpCode, FtpState};

pub fn syst(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    FtpCode::SystemName.send(stream, "UNIX Type: L8").ok()?;
    Some(())
}
