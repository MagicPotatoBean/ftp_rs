use std::{io::Write, net::TcpStream, path::PathBuf};

use crate::{ftp_methods::is_owned, FtpCode, FtpState};

pub fn retr(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        if let (Some(data_con), Some(file)) = (state.data_connection.as_mut(), request) {
            let file_path = state.permission_dir.join(&state.cwd).join(file);
            if !is_owned(&state.permission_dir, &file_path) {
            FtpCode::FileNotFoundOrInvalidPerms.send(stream, "You do not have access to this directory").ok()?;
            return Some(());
            }
            FtpCode::DataConOpenTransferStarting.send(stream, "Transfer started").ok()?;
            if data_con.write_all(b"file_data").is_ok() {
                println!("Sent file");
                FtpCode::RequestCompleted.send(stream, "Fully transferred").ok()?;
            } else {
                println!("Failed to send file");
                FtpCode::ConClosedRequestAborted.send(stream, "Encountered an error").ok()?;
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
fn send(stream: &mut TcpStream, path: PathBuf) {
    if let Ok(mut file) = std::fs::OpenOptions::new().read(true).open(path) {
        loop {
            let mut byte = [0u8];
            match std::io::Read::read(&mut file, &mut byte) {
                Ok(num) => {
                    if num == 0 {
                        break;
                    }
                    stream.write_all(&byte);
                }
                Err(err) => match err.kind() {
                    std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::Interrupted => {}
                    _ => break, // When reached end of file, break.
                },
            }
        }
    } else {
    }
}
