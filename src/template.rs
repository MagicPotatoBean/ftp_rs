use std::net::TcpStream;

use crate::{FtpState, ftp_methods::FtpCode};

pub fn template(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        todo!()
    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
    }
    Some(())
}