use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
mod ftp;
mod http;
fn main() {
    const PASSWORD_SALT: u128 = 662404870180369439363339743;
    let http_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 80));
    let ftp_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 21));
    std::thread::Builder::new().name("FTP Server".to_owned()).spawn(move || ftp::host_server(ftp_addr, 128, PASSWORD_SALT).unwrap()).unwrap();
    std::thread::Builder::new().name("HTTP Server".to_owned()).spawn(move || http::host_server(http_addr, 128, PASSWORD_SALT).unwrap()).unwrap();
    let _ = std::io::stdin().read_line(&mut String::default()); // Block until "\n" received on stdin
}