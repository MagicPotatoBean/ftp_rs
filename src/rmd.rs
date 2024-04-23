use std::net::TcpStream;

use crate::{FtpCode, FtpState, ftp_methods::is_owned};

pub fn rmd(
    stream: &mut TcpStream,
    state: &mut FtpState,
    request: Option<String>,
) -> Option<()> {
    if state.authenticated {
        let mut file_path = state
                .permission_dir
                .join(&state.cwd);
            if let Some(usr_path) = request {
                file_path.push(usr_path);
            }
        if !is_owned(&state.permission_dir, &file_path) {
            FtpCode::FileNotFoundOrInvalidPerms.send(stream, "You do not have access to this directory").ok()?;
            return Some(())
        }
        if std::fs::remove_dir_all(file_path).is_ok() {
            FtpCode::RequestCompleted.send(stream, "Succesfully deleted directory").ok()?;
        } else {
            FtpCode::FileNotFoundOrInvalidPerms.send(stream, "Failed to delete directory").ok()?;
        }
    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
    };
    Some(())
}
