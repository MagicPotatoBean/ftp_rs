use std::net::TcpStream;

use crate::ftp::FtpState;

use super::ftp_methods::FtpCode;

pub fn opts(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        if request == Some("utf8 on".to_string()) {
            FtpCode::CmdNotNeeded.send(stream, "Enabled by default").ok()?
        }
    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
    }
    Some(())
}