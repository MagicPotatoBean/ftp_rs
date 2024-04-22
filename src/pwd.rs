use std::{net::TcpStream, io::Write};

use crate::{FtpResponseCode, FtpState};

pub fn pwd(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        stream.write_all(FtpResponseCode::FileCreated.to_string(&format!("{}{}", state.display_dir, state.cwd.display())).as_bytes()).ok()?;
    } else {
        stream.write_all(FtpResponseCode::NotLoggedIn.to_string("Invalid username or password").as_bytes()).ok()?;
    };
    Some(())
}