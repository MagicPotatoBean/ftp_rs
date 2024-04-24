use std::{
    fs::OpenOptions,
    io::{Read, Write},
    net::TcpStream,
};

use crate::ftp::{ftp_methods::is_owned, FtpCode, FtpState};
use crate::ftp_log;
pub fn retr(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        if let (Some(data_con), Some(file)) = (state.data_connection.as_mut(), request) {
            let file_path = state.permission_dir.join(&state.cwd).join(file);
            if !is_owned(&state.permission_dir, &file_path) {
                FtpCode::FileNotFoundOrInvalidPerms
                    .send(stream, "You do not have access to this directory")
                    .ok()?;
                return Some(());
            }
            FtpCode::DataConOpenTransferStarting
                .send(stream, "Transfer started")
                .ok()?;
            if let Ok(mut file) = OpenOptions::new().read(true).open(&file_path) {
                loop {
                    let mut byte = [0u8];
                    match file.read(&mut byte) {
                        Ok(bytes_read) => {
                            if bytes_read == 0 {
                                break;
                            }
                            if data_con.write_all(&byte).is_err() {
                                FtpCode::ConClosedRequestAborted
                                    .send(stream, "Failed to write entire file.")
                                    .ok()?;
                                return Some(());
                            }
                        }
                        Err(err) => match err.kind() {
                            std::io::ErrorKind::WouldBlock => break,
                            _ => {
                                ftp_log!("Error writing to file: {}", err);
                                return Some(());
                            }
                        },
                    }
                }
                FtpCode::ConClosedRequestSuccess
                    .send(stream, "Succesfully transferred")
                    .ok()?;
            } else {
                FtpCode::FileNotFoundOrInvalidPerms
                    .send(
                        stream,
                        "You do not have permission to access this directory",
                    )
                    .ok()?;
            }
            let _ = data_con.shutdown(std::net::Shutdown::Both);
            state.data_connection = None;
        }
    } else {
        FtpCode::NotLoggedIn
            .send(stream, "Invalid username or password")
            .ok()?;
    }
    Some(())
}