use std::net::TcpStream;

use crate::ftp::{FtpCode, FtpState};

pub fn pwd(stream: &mut TcpStream, state: &mut FtpState, _request: Option<String>) -> Option<()> {
    if state.authenticated {
        FtpCode::FileCreated.send(stream, &format!("{}{}", state.display_dir, state.cwd.display())).ok()?;
    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
    };
    Some(())
}