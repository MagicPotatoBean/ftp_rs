use std::{io::Write, net::TcpStream, path::PathBuf};

use crate::{FtpResponseCode, FtpState};

pub fn mkd(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        let file_path = if let Some(usr_path) = request {
            let path = state
                .permission_dir
                .join(&state.cwd)
                .join(&usr_path);
            if path.starts_with(&state.permission_dir) && !usr_path.contains("..") {
                path
            } else {
                stream
                    .write_all(
                        FtpResponseCode::FileNameDisallowed
                            .to_string("Directory names cannot include \"..\" or \"/\"")
                            .as_bytes(),
                    )
                    .ok()?;
                return Some(());
            }
        } else {
            stream
                .write_all(
                    FtpResponseCode::ParamSyntaxErr
                        .to_string("Must include path parameter")
                        .as_bytes(),
                )
                .ok()?;
            return Some(());
        };
        if std::fs::create_dir(file_path).is_ok() {
            stream
                .write_all(
                    FtpResponseCode::FileCreated
                        .to_string("Succesfully created directory")
                        .as_bytes(),
                )
                .ok()?;
        } else {
            stream
                .write_all(
                    FtpResponseCode::FileNotFoundOrInvalidPerms
                        .to_string("Failed to create directory")
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
    };
    Some(())
}
