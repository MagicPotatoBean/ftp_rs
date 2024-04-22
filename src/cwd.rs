use std::{io::Write, net::TcpStream, path::PathBuf};

use crate::{FtpCode, FtpState};

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
                    FtpCode::RequestCompleted.send(stream, &format!("{}{}", state.display_dir, state.cwd.display())).ok()?;
                } else {
                    FtpCode::FileNotFoundOrInvalidPerms.send(stream, "You dont have permission to enter that folder").ok()?;
                }
                println!("{}", actual_wd.display());
            } else {
                FtpCode::FileNotFoundOrInvalidPerms.send(stream, "You dont have permission to enter that folder").ok()?;
            }
        } else {
            FtpCode::ParamSyntaxErr.send(stream, "Path must be specified").ok()?;
        }
    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
    }
    Some(())
}
