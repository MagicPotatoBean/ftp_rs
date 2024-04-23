use std::net::TcpStream;

use crate::{FtpCode, FtpState};

pub fn user(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    state.authenticated = false;
    state.user = request;
    FtpCode::UnOkNeedPw.send(stream, "Password required").ok()?;
    Some(())
}
