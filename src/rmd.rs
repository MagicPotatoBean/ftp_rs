use std::{net::TcpStream, io::Write};

use crate::{FtpResponseCode, FtpState};

pub fn template(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        let file_path = if let Some(usr_path) = request {
            let path = state
                .permission_dir
                .join(&state.cwd)
                .join(usr_path)
                .canonicalize()
                .ok()?;
            if path.starts_with(&state.permission_dir) {
                path
            } else {
                stream
                    .write_all(
                        FtpResponseCode::FileNotFoundOrInvalidPerms
                            .to_string("Not found or invalid permissions")
                            .as_bytes(),
                    )
                    .ok()?;
                return Some(());
            }
        } else {
            if let Ok(path) = PathBuf::from(&state.permission_dir)
                .join(&state.cwd)
                .canonicalize()
            {
                if path.starts_with(&state.permission_dir) {
                    path
                } else {
                    stream
                        .write_all(
                            FtpResponseCode::FileNotFoundOrInvalidPerms
                                .to_string("Not found or invalid permissions")
                                .as_bytes(),
                        )
                        .ok()?;
                    return Some(());
                }
            } else {
                return Some(());
            }
        };
    } else {
        stream.write_all(FtpResponseCode::NotLoggedIn.to_string("Invalid username or password").as_bytes()).ok()?;
    }
}