use std::net::TcpStream;

use crate::ftp::{FtpCode, FtpState, Types};

pub fn set_type(
    stream: &mut TcpStream,
    state: &mut FtpState,
    request: Option<String>,
) -> Option<()> {
    if state.authenticated {
        if let Some(new_type) = request {
            state.data_type = if let Ok(new_type) = Types::try_from(new_type.as_str()) {
                FtpCode::CmdOk.send(stream, "Changed transfer type").ok()?;
                new_type
            } else {
                FtpCode::ParamSyntaxErr.send(stream, "Unknown requested type").ok()?;
                return Some(());
            }
        }
    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
    }
    Some(())
}
