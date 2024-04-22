use std::{io::Write, net::TcpStream, path::PathBuf};

use crate::{FtpResponseCode, FtpState};

pub fn cwd(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        if let Some(path) = request {
            let cwd = if path.starts_with("/") {
                PathBuf::from("")
            } else {
                state.cwd.join(path)
            };
            if let Ok(actual_wd) = state.permission_dir.join(&cwd).canonicalize() {
                if actual_wd.starts_with(&state.permission_dir) {
                    let abs_path = actual_wd.display().to_string();
                    if !(abs_path.eq(&state.permission_dir.display().to_string())) {
                        state.cwd = PathBuf::from(
                            &abs_path[(state.permission_dir.display().to_string().len() + 1)..],
                        );
                    } else {
                        state.cwd = PathBuf::from("");
                    }
                    stream
                        .write_all(
                            FtpResponseCode::RequestCompleted
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
                println!("{}", actual_wd.display());
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
                    FtpResponseCode::ParamSyntaxErr
                        .to_string("Path must be specified")
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
