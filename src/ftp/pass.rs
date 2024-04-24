use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io::Write,
    net::TcpStream,
    path::PathBuf,
};

use crate::ftp::{FtpCode, FtpState};
use crate::ftp_log;
pub fn pass(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    let mut authname = None;
    match (state.user.clone(), request) {
        (Some(name), Some(pass)) => {
            if name.chars().all(|chr| chr.is_ascii_alphanumeric()) {
                match std::fs::read(PathBuf::from("./static/users").join(&name).join("auth")) {
                    Ok(auth_data) => {
                        let mut hasher = DefaultHasher::new();
                        const SALT: u128 = 662404870180369439363339743;
                        pass.hash(&mut hasher);
                        SALT.hash(&mut hasher);
                        let pass_hash = hasher.finish();
                        if pass_hash.to_be_bytes().to_vec() == auth_data {
                            authname = Some(name);
                        } else {
                            authname = None;
                        }
                    }
                    Err(err) => match err.kind() {
                        std::io::ErrorKind::NotFound => {
                            ftp_log!("New user created: {name}");
                            let mut hasher = DefaultHasher::new();
                            const SALT: u128 = 662404870180369439363339743;
                            pass.hash(&mut hasher);
                            SALT.hash(&mut hasher);
                            let pass_hash = hasher.finish();
                            if std::fs::create_dir(PathBuf::from("./static/users").join(&name))
                                .is_err()
                            {
                                ftp_log!("Cant create user dir");
                                authname = None;
                            } else if std::fs::create_dir(
                                PathBuf::from("./static/users").join(&name).join("files"),
                            )
                            .is_err()
                            {
                                ftp_log!("Cant create user file dir");
                                authname = None;
                            } else {
                                if let Ok(mut auth_file) = std::fs::OpenOptions::new()
                                    .create(true)
                                    .write(true)
                                    .open(PathBuf::from("./static/users").join(&name).join("auth"))
                                {
                                     if auth_file.write_all(&pass_hash.to_be_bytes()).is_ok() {
                                        authname = Some(name)
                                     } else {   
                                        ftp_log!("Couldnt write user data");
                                        authname = None;
                                     }
                                } else {
                                    authname = None;
                                }
                            }
                        }
                        _ => authname = None,
                    },
                }
            }
        }
        _ => {
            authname = None;
        }
    }
    if let Some(name) = authname {
        if state.auth(&name).is_ok() {
            FtpCode::LoggedInProceed.send(stream, "Logged in!").ok()?;
        } else {
            ftp_log!("Failed to call auth");
            FtpCode::NotLoggedIn
                .send(stream, "Invalid username or password")
                .ok()?;
        }
    } else {
        FtpCode::NotLoggedIn
            .send(stream, "Invalid username or password")
            .ok()?;
    }
    Some(())
}
