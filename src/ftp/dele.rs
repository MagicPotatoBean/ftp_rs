use std::net::TcpStream;

use crate::ftp::{FtpState, ftp_methods::{FtpCode, is_owned}};

pub fn dele(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        if let Some(path) = request {
            let file_path = state.permission_dir.join(&state.cwd).join(path);
            if !is_owned(&state.permission_dir, &file_path) {
                FtpCode::FileNotFoundOrInvalidPerms.send(stream, "File not found or invalid permissions").ok()?;
                return Some(())
            }
            if std::fs::remove_file(&file_path).is_ok() {
                FtpCode::RequestCompleted.send(stream, "File deleted").ok()?;
            } else {
                if file_path.exists() {
                    FtpCode::FileNameDisallowed.send(stream, "File name is already taken").ok()?;
                } else {
                    FtpCode::FileNotFoundOrInvalidPerms.send(stream, "You do not have permission to access this directory").ok()?;
                }
            }
        }
    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
    }
    Some(())
}