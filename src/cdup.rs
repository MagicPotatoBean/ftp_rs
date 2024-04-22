use std::{io::Write, net::TcpStream, path::PathBuf};

use crate::{FtpResponseCode, FtpState};

pub fn cdup(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        if state.cwd.pop() {
            stream
                .write_all(
                    FtpResponseCode::CmdOk
                        .to_string(&format!("{}{}", state.display_dir, state.cwd.display()))
                        .as_bytes(),
                )
                .ok()?;
        } else {
            stream
                .write_all(
                    FtpResponseCode::FileNotFoundOrInvalidPerms
                        .to_string("You dont have permission to enter that folder.")
                        .as_bytes(),
                )
                .ok()?;
        }
    } else {
        stream
            .write_all(
                FtpResponseCode::NotLoggedIn
                    .to_string("Invalid username or password")
                    .as_bytes(),
            )
            .ok()?;
    }
    Some(())
}
