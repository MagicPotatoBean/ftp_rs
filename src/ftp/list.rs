use std::{os::unix::{fs::MetadataExt, ffi::OsStrExt}, io::Write, net::TcpStream, ops::Div, path::PathBuf};
use crate::{ftp::{ftp_methods::is_owned, FtpCode, FtpState}, ftp_log};

pub fn list(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        match state.data_connection {
            Some(ref mut data_stream) => {
                
                let mut file_path = state.permission_dir.join(&state.cwd);
                if let Some(usr_dir) = request {
                    file_path.push(usr_dir);
                }
                if !is_owned(&state.permission_dir, &file_path) {
            FtpCode::FileNotFoundOrInvalidPerms.send(stream, "You do not have access to this directory").ok()?;
            return Some(());
                }
                ftp_log!("Datastream is some, writing to it.");
                FtpCode::DataConOpenTransferStarting.send(stream, "Transfer started").ok()?;
                let display_path = if !(file_path
                    .display()
                    .to_string()
                    .eq(&state.permission_dir.display().to_string()))
                {
                    PathBuf::from(
                        &file_path.display().to_string()
                            [(state.permission_dir.display().to_string().len() + 1)..],
                    )
                } else {
                    PathBuf::from("")
                };
                data_stream
                    .write_all(
                        format!("\r\n{}{}\r\n", state.display_dir, display_path.display())
                            .as_bytes(),
                    )
                    .ok()?;
                for file in std::fs::read_dir(file_path).ok()?.flatten() {
                    let len_str = {
                        let len = if let Ok(meta) = file.metadata() {
                            meta.len()
                        } else {
                            continue;
                        };
                        let (chars, symbol) = match len.checked_ilog10().map(|item| item.div(3)) {
                            Some(0) | None => (0, ' '),
                            Some(1) => (1, 'K'),
                            Some(2) => (2, 'M'),
                            Some(3) => (3, 'G'),
                            Some(4) => (4, 'T'),
                            _ => (0, '?'),
                        };
                        let mut str_num =
                            len.to_string()[0..(len.to_string().len() - (chars * 3))].to_owned();
                        if chars != 0 {
                            str_num.push(symbol);
                        }
                        let mut formated_num = " ".repeat(4 - &str_num.len());
                        formated_num.push_str(&str_num);
                        formated_num
                    };
                    let permissions = if let Ok(meta) = file.metadata() {
                        let mut is_dir = if meta.is_file() {
                            "-".to_string()
                        } else {
                            "d".to_string()
                        };
                        let mode = meta.mode();
                        if mode & 0b100000000 == 0b100000000 {
                            is_dir.push('r');
                        } else {
                            is_dir.push('-');
                        }
                        if mode & 0b10000000 == 0b10000000 {
                            is_dir.push('w');
                        } else {
                            is_dir.push('-');
                        }
                        if mode & 0b1000000 == 0b1000000 {
                            is_dir.push('x');
                        } else {
                            is_dir.push('-');
                        }
                        if mode & 0b100000 == 0b100000 {
                            is_dir.push('r');
                        } else {
                            is_dir.push('-');
                        }
                        if mode & 0b10000 == 0b10000 {
                            is_dir.push('w');
                        } else {
                            is_dir.push('-');
                        }
                        if mode & 0b1000 == 0b1000 {
                            is_dir.push('x');
                        } else {
                            is_dir.push('-');
                        }
                        if mode & 0b100 == 0b100 {
                            is_dir.push('r');
                        } else {
                            is_dir.push('-');
                        }
                        if mode & 0b10 == 0b10 {
                            is_dir.push('w');
                        } else {
                            is_dir.push('-');
                        }
                        if mode & 0b1 == 0b1 {
                            is_dir.push('x');
                        } else {
                            is_dir.push('-');
                        }
                        is_dir
                    } else {
                        continue;
                    };
                    let date_str = {
                        if let Ok(meta) = file.metadata() {
                            if let Ok(modified_time) = meta.modified() {
                                let local_time: chrono::prelude::DateTime<chrono::Local> =
                                    chrono::DateTime::from(modified_time);
                                local_time.format("%b %d %H:%M").to_string()
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    };
                    if data_stream.write_all(permissions.as_bytes()).is_err() ||
                    data_stream.write_all(b" ").is_err() ||
                    data_stream.write_all(len_str.as_bytes()).is_err() ||
                    data_stream.write_all(b" ").is_err() ||
                    data_stream.write_all(date_str.as_bytes()).is_err() ||
                    data_stream.write_all(b" ").is_err() ||
                    data_stream.write_all(file.file_name().as_bytes()).is_err() ||
                    data_stream.write_all(b"\r\n").is_err() {
                        FtpCode::ConClosedRequestAborted.send(stream, "Failed to send file data").ok()?;
                        return Some(());
                    }
                }
                if data_stream.write_all(b"\r\n").is_err() ||
                data_stream.shutdown(std::net::Shutdown::Both).is_err() {
                    FtpCode::ConClosedRequestAborted.send(stream, "Failed to close data stream").ok()?;
                    return Some(());
                }
                state.data_connection = None;
                FtpCode::ConClosedRequestSuccess.send(stream, "Success").ok()?;
            }
            None => {
                ftp_log!("Datastream is none, informing user.");
                FtpCode::CantOpenDataCon.send(stream, "Data connection wasnt open").ok()?;
            }
        }
    } else {
        FtpCode::NotLoggedIn
            .send(stream, "Invalid username or password")
            .ok()?;
    };
    Some(())
}
