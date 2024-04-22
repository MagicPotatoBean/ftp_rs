use std::{net::TcpStream, io::Write};

use crate::{FtpResponseCode, FtpState};

pub fn template(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {

    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
    }
}