use std::{
    fmt::Display,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    path::{Path, PathBuf},
    str::FromStr,
};
use ftp_methods::{FtpMethod, FtpCode, FtpPacket};
mod ftp_methods;
mod cdup;
mod cwd;
mod handshake;
mod help;
mod list;
mod mkd;
mod pass;
mod port;
mod pwd;
mod quit;
mod rein;
mod retr;
mod set_type;
mod syst;
mod user;
mod stor;
mod rmd;
fn main() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 65432))
        .expect("Failed to bind to port.");
    for client in listener.incoming() {
        println!("Got tcp connection.");
        match client {
            Ok(mut stream) => {
                handshake::handshake(&mut stream);
                let mut state: FtpState =
                    FtpState::new("./static").display_dir("[root@nixos]:/".to_string());
                loop {
                    match read_request(&mut stream) {
                        Ok(request) => {
                            println!("Request: {request}");
                            let should_disconnect = match request.method {
                                FtpMethod::User => {
                                    user::user(&mut stream, &mut state, request.data)
                                }
                                FtpMethod::Pass => {
                                    pass::pass(&mut stream, &mut state, request.data)
                                }
                                FtpMethod::Syst => {
                                    syst::syst(&mut stream, &mut state, request.data)
                                }
                                FtpMethod::Port => {
                                    port::port(&mut stream, &mut state, request.data)
                                }
                                FtpMethod::List => {
                                    list::list(&mut stream, &mut state, request.data)
                                }
                                FtpMethod::Cwd => cwd::cwd(&mut stream, &mut state, request.data),
                                FtpMethod::Cdup => {
                                    cdup::cdup(&mut stream, &mut state, request.data)
                                }
                                FtpMethod::Help => {
                                    help::help(&mut stream, &mut state, request.data)
                                }
                                FtpMethod::Type => {
                                    set_type::set_type(&mut stream, &mut state, request.data)
                                }
                                FtpMethod::Retr => {
                                    retr::retr(&mut stream, &mut state, request.data)
                                }
                                FtpMethod::Quit => {
                                    quit::quit(&mut stream, &mut state, request.data)
                                }
                                FtpMethod::Mkd => mkd::mkd(&mut stream, &mut state, request.data),
                                FtpMethod::Pwd => pwd::pwd(&mut stream, &mut state, request.data),
                                FtpMethod::Rmd => rmd::rmd(&mut stream, &mut state, request.data),
                                FtpMethod::Stor => stor::stor(&mut stream, &mut state, request.data),
                                method => stream
                                    .write_all(
                                        FtpCode::CmdNotImpl
                                            .to_string(&format!(
                                                "\"{}\" is not implemented yet.",
                                                method.to_string()
                                            ))
                                            .as_bytes(),
                                    )
                                    .ok(),
                            }
                            .is_none();
                            if should_disconnect {
                                println!("Forced to disconnect");
                                break;
                            }
                        }
                        Err(err) => {
                            println!("Request failed: \"{err}\"");
                            stream.write_all(
                                FtpCode::CmdNotImpl
                                    .to_string("Not implemented")
                                    .as_bytes(),
                            );
                        }
                    }
                }
                // read_to_end(&mut stream);
            }
            Err(_) => println!("Failed to connect."),
        }
    }
}
struct FtpState {
    user: Option<String>,
    authenticated: bool,
    cwd: PathBuf,
    permission_dir: PathBuf,
    display_dir: String,
    data_connection: Option<TcpStream>,
    data_type: Types,
}
impl FtpState {
    fn new<T: AsRef<Path>>(permission_dir: T) -> Self {
        FtpState {
            cwd: PathBuf::from_str("").expect("Infallible operation"),
            data_connection: None,
            user: None,
            authenticated: false,
            permission_dir: permission_dir.as_ref().canonicalize().unwrap(),
            display_dir: "/".to_owned(),
            data_type: Types::ASCII,
        }
    }
    fn cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = cwd.canonicalize().unwrap();
        self
    }
    fn display_dir(mut self, dir: String) -> Self {
        self.display_dir = dir;
        self
    }
}
impl Default for FtpState {
    fn default() -> Self {
        FtpState {
            cwd: PathBuf::from_str("").expect("Infallible operation"),
            data_connection: None,
            user: None,
            authenticated: false,
            permission_dir: PathBuf::from("./static").canonicalize().unwrap(),
            display_dir: "/".to_owned(),
            data_type: Types::ASCII,
        }
    }
}
enum Types {
    ASCII,
    EBCDIC,
    Image,
}
impl TryFrom<&str> for Types {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "I" => Ok(Self::Image),
            "A" => Ok(Self::ASCII),
            "E" => Ok(Self::EBCDIC),
            _ => Err(()),
        }
    }
}
fn read_request(stream: &mut TcpStream) -> Result<FtpPacket, String> {
    let raw = read_until_consume(&stream, '\r');
    stream.read(&mut [0u8]).map_err(|err| err.to_string())?;
    FtpPacket::new(String::from_utf8_lossy(&raw).to_string())
        .ok_or(String::from_utf8_lossy(&raw).to_string())
}
fn read_until_consume(mut stream: &TcpStream, chr: char) -> Vec<u8> {
    let mut byte = [0u8];
    let mut output = Vec::new();
    let _delay = stream.read_timeout().unwrap();
    loop {
        match stream.read(&mut byte).map_err(|err| err.kind()) {
            Ok(size) => {
                if size == 0 {
                    break output;
                }
                if char::from(byte[0]) == chr {
                    break output;
                }
                output.push(byte[0]);
            }
            Err(err) => {
                println!("Encountered an error: {err}");
            }
        }
    }
}