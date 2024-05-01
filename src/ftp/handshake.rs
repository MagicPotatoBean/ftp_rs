use std::{net::TcpStream, io::Write};

use crate::ftp::FtpCode;

pub fn handshake(stream: &mut TcpStream) -> Option<()> {
    let message = r#"
=======================================
Hello! This is an RFC-959 compliant FTP
server. If you do not have an account,
just try login with a new username and
password, and your account should be 
created and logged in as though you 
always had an account.
=======================================
 "#;
    let last_index = message.lines().count() - 1;
    for (index, mut line) in message.lines().enumerate() {
        let mut out = "".to_string();
        if index == 0 {
            out.push_str("200-");
        }
        if index == last_index {
            out.push_str("200 ");
        }
        out.push_str(line);
        out.push_str("\r\n");
        crate::ftp_log!("Response: {}", out);
        stream.write_all(out.as_bytes()).ok()?;
    }

    Some(())
}
