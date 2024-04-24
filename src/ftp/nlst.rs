use std::{io::Write, net::TcpStream, os::unix::ffi::OsStrExt, path::PathBuf};

use crate::ftp::{
    ftp_methods::{is_owned, FtpCode},
    FtpState,
};
use crate::ftp_log;

pub fn nlst(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        match state.data_connection {
            Some(ref mut data_stream) => {
                let mut file_path = state.permission_dir.join(&state.cwd);
                if let Some(usr_dir) = request {
                    file_path.push(usr_dir);
                }
                if !is_owned(&state.permission_dir, &file_path) {
                    FtpCode::FileNotFoundOrInvalidPerms
                        .send(stream, "You do not have access to this directory")
                        .ok()?;
                    return Some(());
                }
                ftp_log!("Datastream is some, writing to it.");
                if let Ok(dir) = std::fs::read_dir(file_path) {
                    FtpCode::DataConOpenTransferStarting
                        .send(stream, "Transfer started")
                        .ok()?;
                    ftp_log!("Sending files");
                    for file in dir.flatten() {
                        if let Ok(meta) = file.metadata() {
                            if meta.is_file() {
                                ftp_log!("Printing file {}", file.file_name().to_string_lossy());
                                if data_stream.write_all(file.file_name().as_bytes()).is_err() {
                                    FtpCode::ConClosedRequestAborted
                                    .send(stream, "Failed to write to data connection")
                                    .ok()?;
                                return Some(());
                                }
                                if data_stream.write_all(b"\r\n").is_err() {
                                    FtpCode::ConClosedRequestAborted
                                        .send(stream, "Failed to write to data connection")
                                        .ok()?;
                                    return Some(());
                                }
                            }
                        }
                    }
                    ftp_log!("Sent all files");
                    if data_stream.shutdown(std::net::Shutdown::Both).is_err() {
                        FtpCode::ConClosedRequestAborted
                            .send(stream, "Failed to close data connection")
                            .ok()?;
                        state.data_connection = None;
                        return Some(());
                    } else {
                        state.data_connection = None;
                    }
                    FtpCode::RequestCompleted
                        .send(stream, "Succesfully transferred")
                        .ok()?;
                } else {
                    FtpCode::FileNotFoundOrInvalidPerms
                        .send(stream, "Failed to read directory")
                        .ok()?;
                }
            }
            None => {
                ftp_log!("Datastream is none, informing user.");
                FtpCode::CantOpenDataCon
                    .send(stream, "Data connection wasnt open")
                    .ok()?;
            }
        }
    } else {
        FtpCode::NotLoggedIn
            .send(stream, "Invalid username or password")
            .ok()?;
    }
    Some(())
}
