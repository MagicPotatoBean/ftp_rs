use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream, SocketAddr},
    path::{Path, PathBuf},
    str::FromStr, time::Duration, sync::Arc, thread
};
use crate::ftp_log;
// use chrono::Duration;
use ftp_methods::{FtpMethod, FtpCode, FtpPacket};
pub mod ftp_methods;
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
mod dele;
mod nlst;
pub fn host_server(address: SocketAddr, max_threads: usize, salt: u128) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    let thread_count: Arc<()> = Arc::new(()); // Counts the number of threads spawned based on the weak count
    ftp_log!("==================== FTP Server running on {address} ====================");
    for client in listener.incoming().flatten() {
        if Arc::strong_count(&thread_count) <= max_threads {
            /* Ignores request if too many threads are spawned */
            let passed_count = thread_count.clone();
            if thread::Builder::new()
                .name("ClientHandler".to_string())
                .spawn(move || handle_connection(passed_count, client, salt))
                .is_err()
            {
                /* Spawn thread to handle request */
                ftp_log!("Failed to spawn thread");
            }
        }
    }

    drop(thread_count);
    Ok(())
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
    fn auth(&mut self, username: &str) -> std::io::Result<()> {
        self.display_dir = format!("[{username}]:/");
        self.permission_dir = PathBuf::from("./static/users").join(username).join("files").canonicalize()?;
        self.authenticated = true;
        Ok(())
    }
    fn deauth(&mut self) -> std::io::Result<()> {
        self.display_dir = "[anonymous]:/".to_string();
        self.permission_dir = PathBuf::from("./static/unauth").canonicalize()?;
        self.authenticated = false;
        Ok(())
    }
    // fn cwd(mut self, cwd: PathBuf) -> Self {
    //     self.cwd = cwd.canonicalize().unwrap();
    //     self
    // }
    fn display_dir(mut self, dir: String) -> Self {
        self.display_dir = dir;
        self
    }
}
impl Default for FtpState {
    fn default() -> Self {
        FtpState {
            cwd: PathBuf::from(""),
            data_connection: None,
            user: None,
            authenticated: false,
            permission_dir: PathBuf::from("./static/unauth").canonicalize().unwrap(),
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
                ftp_log!("Encountered an error: {err}");
            }
        }
    }
}
fn handle_connection(thread_counter: Arc<()>, mut stream: TcpStream, salt: u128) {
    {
        handshake::handshake(&mut stream);
        let mut state: FtpState =
            FtpState::default().display_dir("[anonymous]:/".to_string());
        loop {
            match read_request(&mut stream) {
                Ok(request) => {
                    ftp_log!("Request: {request}");
                    let should_disconnect = match request.method {
                        FtpMethod::User => {
                            user::user(&mut stream, &mut state, request.data)
                        }
                        FtpMethod::Pass => {
                            pass::pass(&mut stream, &mut state, request.data, salt)
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
                        FtpMethod::Dele => dele::dele(&mut stream, &mut state, request.data),
                        FtpMethod::Rein => rein::rein(&mut stream, &mut state, request.data),
                        FtpMethod::Nlst => nlst::nlst(&mut stream, &mut state, request.data),
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
                        ftp_log!("Forced to disconnect");
                        break;
                    }
                }
                Err(err) => {
                    ftp_log!("Request failed: \"{err}\"");
                    if FtpCode::CmdNotImpl.send(&mut stream, "Not implemented.").is_err() {
                        break;
                    }
                }
            }
        }
        // read_to_end(&mut stream);
        drop(thread_counter);
    }
}
#[macro_export]
macro_rules! ftp_log {
    () => {
        use std::io::Write;
        let current_time: DateTime<Utc> = Utc::now();
        std::fs::OpenOptions::new().append(true).open("ftp.log").expect("Failed to open ftp.log file").write_all(format!("[{} UTC] {}:{}:{}\n", current_time.format("%Y-%m-%d %H:%M:%S"), file!(), line!(), column!()).as_bytes()).expect("Failed to write to log file");
        println!("[{} UTC] {}:{}:{}", current_time.format("%Y-%m-%d %H:%M:%S"), file!(), line!(), column!());
    };
    ($($arg:tt)*) => {{
        use std::io::Write;
        let current_time: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
        std::fs::OpenOptions::new().append(true).open("ftp.log").expect("Failed to open ftp.log file").write_all(format!("[{} UTC] {}:{}:{}: {}\n", current_time.format("%Y-%m-%d %H:%M:%S"), file!(), line!(), column!(), format!($($arg)*)).as_bytes()).expect("Failed to write to log file");
        println!("[{} UTC] {}:{}:{}: {}", current_time.format("%Y-%m-%d %H:%M:%S"), file!(), line!(), column!(), format!($($arg)*));
    }};
}
