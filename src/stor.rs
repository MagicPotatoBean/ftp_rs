use std::{net::TcpStream, io::Write};

use crate::{FtpCode, FtpState};

pub fn stor(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        todo!()
    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
    };
    todo!()
}