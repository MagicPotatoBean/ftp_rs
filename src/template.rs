use std::{net::TcpStream, io::Write};

use crate::{FtpResponseCode, FtpState};

pub fn template(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {

    } else {
        stream.write_all(FtpResponseCode::NotLoggedIn.to_string("Invalid username or password").as_bytes()).ok()?;
    }
}