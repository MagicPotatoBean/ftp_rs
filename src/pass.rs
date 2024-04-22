use std::{io::Write, net::TcpStream};

use crate::{FtpCode, FtpState};

pub fn pass(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    match (state.user.as_ref(), request) {
        (Some(name), Some(pass)) => {
            if pass == "toor" && name == "root" {
                state.authenticated = true;
            } else {
                state.authenticated = false;
            }
        }
        _ => {
            state.authenticated = false;
        }
    }
    if state.authenticated {
        FtpCode::LoggedInProceed.send(stream, "Logged in!").ok()?;
    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
    }
    Some(())
}
