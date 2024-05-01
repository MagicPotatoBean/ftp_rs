use std::{net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream}, ops::{Div, Rem}};

use crate::ftp::{ftp_methods::FtpCode, FtpState};

use super::list;

pub fn pasv(
    stream: &mut TcpStream,
    state: &mut FtpState,
    request: Option<String>,
    mut pub_addr: SocketAddr,
    mut priv_addr: SocketAddr,
) -> Option<()> {
    if state.authenticated {
        for port in 1025..2000 {
            priv_addr.set_port(1025);
            if let Ok(listener) = TcpListener::bind(priv_addr) {
                if let IpAddr::V4(ipv4) = pub_addr.ip() {
                    let [ip1, ip2, ip3, ip4] = ipv4.octets();
                    let (port1, port2) = (priv_addr.port().div(256), priv_addr.port().rem(256));
                    FtpCode::EnteringPassive.send(stream, &format!("({},{},{},{},{},{})", ip1, ip2, ip3, ip4, port1, port2)).ok()?;
                    if let Ok((connection, _)) = listener.accept() {
                        state.data_connection = Some(connection);
                        FtpCode::DataConOpenNoTransfer.send(stream, "Connected").ok()?;
                        return Some(());
                    } else {
                        FtpCode::CmdSyntaxErr.send(stream, "UH OH").ok()?;
                        return Some(());
                    }
                }
            }
        }
    }
    Some(())
}
