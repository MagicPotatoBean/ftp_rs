use std::{io::Write, net::TcpStream, path::PathBuf};

use crate::{FtpCode, FtpState, ftp_methods::is_owned};

pub fn mkd(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        let mut file_path = state.permission_dir.join(&state.cwd);
        if let Some(usr_val) = request {
            file_path.push(usr_val);
        }
        if !is_owned(&state.permission_dir, &file_path) {
            FtpCode::FileNotFoundOrInvalidPerms.send(stream, "You do not have access to this directory").ok()?;
            return Some(());
        }
        if std::fs::create_dir(file_path).is_ok() {
            FtpCode::FileCreated.send(stream, "Succesfully created directory").ok()?;
        } else {
            FtpCode::FileNotFoundOrInvalidPerms.send(stream, "Couldnt create directory").ok()?;
        }
    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
    };
    Some(())
}
