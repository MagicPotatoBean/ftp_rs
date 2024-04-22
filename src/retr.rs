use std::{io::Write, net::TcpStream};

use crate::{FtpResponseCode, FtpState};

pub fn retr(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        if let (Some(data_con), Some(file)) = (state.data_connection.as_mut(), request) {
            stream
                .write_all(
                    FtpResponseCode::DataConOpenTransferStarting
                        .to_string("Transfer started")
                        .as_bytes(),
                )
                .ok()?;
            if let Ok(requested_file) = state
                .permission_dir
                .join(&state.cwd)
                .join(file)
                .canonicalize()
            {
                if requested_file.starts_with(&state.permission_dir) {
                    if data_con.write_all(b"file_data").is_ok() {
                        println!("Sent file");
                        stream
                            .write_all(
                                FtpResponseCode::RequestCompleted
                                    .to_string("Fully transferred")
                                    .as_bytes(),
                            )
                            .ok()?;
                    } else {
                        println!("Failed to send file");
                        stream
                            .write_all(
                                FtpResponseCode::ConClosedRequestAborted
                                    .to_string("Encountered an error")
                                    .as_bytes(),
                            )
                            .ok()?;
                    }
                    let _ = data_con.shutdown(std::net::Shutdown::Both);
                    state.data_connection = None;
                } else {
                    println!("Invalid perms");
                    stream
                        .write_all(
                            FtpResponseCode::FileNotFoundOrInvalidPerms
                                .to_string("Not found or invalid permissions")
                                .as_bytes(),
                        )
                        .ok()?;
                }
            } else {
                stream
                    .write_all(
                        FtpResponseCode::FileNotFoundOrInvalidPerms
                            .to_string("Not found or invalid permissions")
                            .as_bytes(),
                    )
                    .ok()?;
            }
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
