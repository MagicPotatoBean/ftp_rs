use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    str::FromStr,
};

use crate::{FtpResponseCode, FtpState};

pub fn port(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        println!("Attempting to change port.");
        let addr = decompose_port(&request?)?;
        match TcpStream::connect(addr) {
            Ok(new_stream) => {
                stream
                    .write_all(
                        FtpResponseCode::CmdOk
                            .to_string(&format!("Opened data connection on {}", addr))
                            .as_bytes(),
                    )
                    .ok()?;
                state.data_connection = Some(new_stream);
            }
            Err(_) => {
                stream
                    .write_all(
                        FtpResponseCode::CantOpenDataCon
                            .to_string("Failed to open data connection")
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
fn decompose_port(data: &str) -> Option<SocketAddrV4> {
    let mut chunks = data.split(",");
    let ip_parts: Vec<String> = chunks
        .by_ref()
        .take(4)
        .map(|item| item.to_owned())
        .collect();
    let mut ip_str = ip_parts.get(0)?.clone();
    ip_str.push('.');
    ip_str.push_str(&ip_parts.get(1)?);
    ip_str.push('.');
    ip_str.push_str(&ip_parts.get(2)?);
    ip_str.push('.');
    ip_str.push_str(&ip_parts.get(3)?);
    let ip = Ipv4Addr::from_str(&ip_str).ok()?;
    let port_parts: Vec<String> = chunks.take(2).map(|item| item.to_owned()).collect();
    let (a, b): (u16, u16) = (port_parts[0].parse().ok()?, port_parts[1].parse().ok()?);
    let port = a * 256 as u16 + b;
    Some(SocketAddrV4::new(ip, port))
}
