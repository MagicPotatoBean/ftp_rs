use std::{io::Write, net::TcpStream};

use crate::{FtpResponseCode, FtpState};

pub fn user(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    state.authenticated = false;
    state.user = request;
    stream
        .write_all(
            FtpResponseCode::UnOkNeedPw
                .to_string("Password required")
                .as_bytes(),
        )
        .ok()?;
    Some(())
}
