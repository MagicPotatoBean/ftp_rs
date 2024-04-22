use std::{io::Write, net::TcpStream};

use crate::{FtpCode, FtpState};

pub fn rein(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    *state = FtpState::default();
    Some(())
}
