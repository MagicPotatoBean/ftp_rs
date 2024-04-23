use std::{net::TcpStream, io::{Write, Read}, fs::OpenOptions};

use crate::{FtpCode, FtpState, ftp_methods::is_owned};

pub fn stor(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        if let Some(ref mut data_con) = state.data_connection {
            if let Some(request_dir) = request {
                let file_path = state.permission_dir.join(&state.cwd).join(request_dir);
                if !is_owned(&state.permission_dir, &file_path) {
                    FtpCode::FileNotFoundOrInvalidPerms.send(stream, "You do not have permission to acces that directory").ok()?;
                    return Some(())
                }
                FtpCode::DataConOpenTransferStarting.send(stream, "Starting transfer").ok()?;
                // Transfer here
                if let Ok(mut file) = OpenOptions::new().create_new(true).write(true).open(&file_path) {
                    loop {
                        let mut byte = [0u8];
                        match data_con.read(&mut byte) {
                            Ok(bytes_read) => {
                                if bytes_read == 0 {
                                    break;
                                }
                                if file.write_all(&byte).is_err() {
                                    FtpCode::ConClosedRequestAborted.send(stream, "Failed to write entire file.").ok()?;
                                    return Some(())
                                }
                            },
                            Err(err) => {
                                match err.kind() {
                                    std::io::ErrorKind::WouldBlock => {break},
                                    _ => {
                                        println!("Error writing to file: {}", err);
                                        return Some(())
                                    }
                                }
                            },
                        }
                    }
                    FtpCode::ConClosedRequestSuccess.send(stream, "Succesfully transferred").ok()?;
                } else {
                    if file_path.exists() {
                        FtpCode::FileNameDisallowed.send(stream, "File name is already taken").ok()?;
                    } else {
                        FtpCode::FileNotFoundOrInvalidPerms.send(stream, "You do not have permission to access this directory").ok()?;
                    }
                }
            } else {
                FtpCode::ParamSyntaxErr.send(stream, "Requires a file path").ok()?;
            }
        } else {
            FtpCode::CantOpenDataCon.send(stream, "Data connection not open").ok()?;
        }
    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
    };
    Some(())
}