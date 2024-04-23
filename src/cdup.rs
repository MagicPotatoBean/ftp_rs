use std::net::TcpStream;

use crate::{FtpCode, FtpState};

pub fn cdup(stream: &mut TcpStream, state: &mut FtpState, _request: Option<String>) -> Option<()> {
    if state.authenticated {
        if state.cwd.pop() {
            FtpCode::CmdOk
                .send(
                    stream,
                    &format!("{}{}", state.display_dir, state.cwd.display()),
                )
                .ok()?;
        } else {
            FtpCode::FileNotFoundOrInvalidPerms
                .send(stream, "You dont have permission to enter that folder")
                .ok()?;
        }
    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
    }
    Some(())
}
