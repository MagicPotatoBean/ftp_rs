use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    str::FromStr, time::Duration,
};

use crate::ftp::{FtpCode, FtpState};
use crate::ftp_log;
pub fn port(stream: &mut TcpStream, state: &mut FtpState, request: Option<String>) -> Option<()> {
    if state.authenticated {
        ftp_log!("Attempting to change port.");
        let addr = decompose_port(&request?)?;
        match TcpStream::connect_timeout(&std::net::SocketAddr::V4(addr), Duration::from_secs(5)) {
            Ok(new_stream) => {
                FtpCode::CmdOk.send(stream, &format!("Opened data connection on {}", addr)).ok()?;
                state.data_connection = Some(new_stream);
            }
            Err(_) => {
                FtpCode::CantOpenDataCon.send(stream, "Failed to open data connection").ok()?;
                return Some(());
            }
        }
    } else {
        FtpCode::NotLoggedIn.send(stream, "Invalid username or password").ok()?;
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
    let (a, b): (u16, u16) = (port_parts.get(0)?.parse().ok()?, port_parts.get(1)?.parse().ok()?);
    let port = a * 256 as u16 + b;
    Some(SocketAddrV4::new(ip, port))
}
