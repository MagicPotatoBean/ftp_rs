use std::{io::Write, net::TcpStream};

use crate::{FtpResponseCode, FtpState, Types};

pub fn set_type(
    stream: &mut TcpStream,
    state: &mut FtpState,
    request: Option<String>,
) -> Option<()> {
    if state.authenticated {
        if let Some(new_type) = request {
            state.data_type = if let Ok(new_type) = Types::try_from(new_type.as_str()) {
                stream
                    .write_all(
                        FtpResponseCode::CmdOk
                            .to_string("Changed transfer type")
                            .as_bytes(),
                    )
                    .ok()?;
                new_type
            } else {
                stream
                    .write_all(
                        FtpResponseCode::ParamSyntaxErr
                            .to_string("Unknown requested type")
                            .as_bytes(),
                    )
                    .ok()?;
                return Some(());
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
