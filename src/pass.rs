use std::{io::Write, net::TcpStream};

use crate::{FtpResponseCode, FtpState};

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
        stream
            .write_all(
                FtpResponseCode::LoggedInProceed
                    .to_string("Logged in.")
                    .as_bytes(),
            )
            .ok()?;
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
