use std::{net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream}, ops::{Div, Rem}};

use crate::ftp::{ftp_methods::FtpCode, FtpState};

use super::list;

pub fn template(
    stream: &mut TcpStream,
    state: &mut FtpState,
    request: Option<String>,
    mut addr: SocketAddr,
) -> Option<()> {
    if state.authenticated {
        for port in 1025..2000 {
            addr.set_port(1025);
            if let Ok(listener) = TcpListener::bind(addr) {
                if let IpAddr::V4(ipv4) = addr.ip() {
                    let [ip1, ip2, ip3, ip4] = ipv4.octets();
                    let (port1, port2) = (addr.port().div(256), addr.port().rem(256));
                    FtpCode::EnteringPassive.send(stream, &format!("({},{},{},{},{},{})", ip1, ip2, ip3, ip4, port1, port2)).ok()?;
                    if let Ok((connection, _)) = listener.accept() {
                        state.data_connection = Some(connection);
                    }
                }
            } else {
                FtpCode::NotLoggedIn
                    .send(stream, "Invalid username or password")
                    .ok()?;
            }
        }
    }
    Some(())
}
