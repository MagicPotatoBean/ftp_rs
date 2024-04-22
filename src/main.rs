use std::{
    fmt::Display,
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream},
    path::{Path, PathBuf},
    str::FromStr,
};
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
                                method => stream
                                    .write_all(
                                        FtpResponseCode::CmdNotImpl
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
                                FtpResponseCode::CmdNotImpl
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
enum FtpResponseCode {
    CmdOk,
    CmdSyntaxErr,
    ParamSyntaxErr,
    CmdNotNeeded,
    CmdNotImpl,
    BadSequence,
    CmdNotImplForParam,
    RestartMarker,
    SystStatusOrHelp,
    DirStatus,
    FileStatus,
    HelpMsg,
    SystemName,
    ReadyInNMinutes,
    ReadyForNewUser,
    ServiceClosing,
    ServiceNotAvailable,
    DataConOpenTransferStarting,
    DataConOpenNoTransfer,
    CantOpenDataCon,
    ConClosedRequestSuccess,
    ConClosedRequestAborted,
    EnteringPassive,
    LoggedInProceed,
    NotLoggedIn,
    UnOkNeedPw,
    NeedAcctForLogin,
    NeedAcctForFiles,
    FileOkOpeningDataCon,
    RequestCompleted,
    FileCreated,
    RequestedFileActionRequiresInfo,
    FileBusy,
    FileNotFoundOrInvalidPerms,
    RequestAbortedLocalErr,
    RequestAbortedPageErr,
    InsufficientStorage,
    ExceededStorageAllocation,
    FileNameDisallowed,
}
impl FtpResponseCode {
    fn to_string(&self, msg: &str) -> String {
        let mut code = match self {
            FtpResponseCode::CmdOk => 200,
            FtpResponseCode::CmdSyntaxErr => 500,
            FtpResponseCode::ParamSyntaxErr => 501,
            FtpResponseCode::CmdNotNeeded => 202,
            FtpResponseCode::CmdNotImpl => 502,
            FtpResponseCode::BadSequence => 503,
            FtpResponseCode::CmdNotImplForParam => 504,
            FtpResponseCode::RestartMarker => 110,
            FtpResponseCode::SystStatusOrHelp => 211,
            FtpResponseCode::DirStatus => 212,
            FtpResponseCode::FileStatus => 213,
            FtpResponseCode::HelpMsg => 214,
            FtpResponseCode::SystemName => 215,
            FtpResponseCode::ReadyInNMinutes => 120,
            FtpResponseCode::ReadyForNewUser => 220,
            FtpResponseCode::ServiceClosing => 221,
            FtpResponseCode::ServiceNotAvailable => 421,
            FtpResponseCode::DataConOpenTransferStarting => 125,
            FtpResponseCode::DataConOpenNoTransfer => 225,
            FtpResponseCode::CantOpenDataCon => 425,
            FtpResponseCode::ConClosedRequestSuccess => 226,
            FtpResponseCode::ConClosedRequestAborted => 426,
            FtpResponseCode::EnteringPassive => 227,
            FtpResponseCode::LoggedInProceed => 230,
            FtpResponseCode::NotLoggedIn => 530,
            FtpResponseCode::UnOkNeedPw => 331,
            FtpResponseCode::NeedAcctForLogin => 332,
            FtpResponseCode::NeedAcctForFiles => 532,
            FtpResponseCode::FileOkOpeningDataCon => 150,
            FtpResponseCode::RequestCompleted => 250,
            FtpResponseCode::FileCreated => 257,
            FtpResponseCode::RequestedFileActionRequiresInfo => 350,
            FtpResponseCode::FileBusy => 450,
            FtpResponseCode::FileNotFoundOrInvalidPerms => 550,
            FtpResponseCode::RequestAbortedLocalErr => 451,
            FtpResponseCode::RequestAbortedPageErr => 551,
            FtpResponseCode::InsufficientStorage => 452,
            FtpResponseCode::ExceededStorageAllocation => 552,
            FtpResponseCode::FileNameDisallowed => 553,
        }
        .to_string();
        if msg.len() >= 1 {
            code.push(' ');
            code.push_str(msg);
        }
        code.push('\n');
        code
    }
    fn from_string(string: &str) -> Option<Self> {
        Some(match string {
            "200" => FtpResponseCode::CmdOk,
            "500" => FtpResponseCode::CmdSyntaxErr,
            "501" => FtpResponseCode::ParamSyntaxErr,
            "202" => FtpResponseCode::CmdNotNeeded,
            "502" => FtpResponseCode::CmdNotImpl,
            "503" => FtpResponseCode::BadSequence,
            "504" => FtpResponseCode::CmdNotImplForParam,
            "110" => FtpResponseCode::RestartMarker,
            "211" => FtpResponseCode::SystStatusOrHelp,
            "212" => FtpResponseCode::DirStatus,
            "213" => FtpResponseCode::FileStatus,
            "214" => FtpResponseCode::HelpMsg,
            "215" => FtpResponseCode::SystemName,
            "120" => FtpResponseCode::ReadyInNMinutes,
            "220" => FtpResponseCode::ReadyForNewUser,
            "221" => FtpResponseCode::ServiceClosing,
            "421" => FtpResponseCode::ServiceNotAvailable,
            "125" => FtpResponseCode::DataConOpenTransferStarting,
            "225" => FtpResponseCode::DataConOpenNoTransfer,
            "425" => FtpResponseCode::CantOpenDataCon,
            "226" => FtpResponseCode::ConClosedRequestSuccess,
            "426" => FtpResponseCode::ConClosedRequestAborted,
            "227" => FtpResponseCode::EnteringPassive,
            "230" => FtpResponseCode::LoggedInProceed,
            "530" => FtpResponseCode::NotLoggedIn,
            "331" => FtpResponseCode::UnOkNeedPw,
            "332" => FtpResponseCode::NeedAcctForLogin,
            "532" => FtpResponseCode::NeedAcctForFiles,
            "150" => FtpResponseCode::FileOkOpeningDataCon,
            "250" => FtpResponseCode::RequestCompleted,
            "257" => FtpResponseCode::FileCreated,
            "350" => FtpResponseCode::RequestedFileActionRequiresInfo,
            "450" => FtpResponseCode::FileBusy,
            "550" => FtpResponseCode::FileNotFoundOrInvalidPerms,
            "451" => FtpResponseCode::RequestAbortedLocalErr,
            "551" => FtpResponseCode::RequestAbortedPageErr,
            "452" => FtpResponseCode::InsufficientStorage,
            "552" => FtpResponseCode::ExceededStorageAllocation,
            "553" => FtpResponseCode::FileNameDisallowed,
            _ => return None,
        })
    }
}
struct FtpPacket {
    method: FtpMethod,
    data: Option<String>,
}
impl FtpPacket {
    fn new(str: String) -> Option<Self> {
        if let Some((method, value)) = str.split_once(" ") {
            Some(Self {
                method: FtpMethod::try_from(method.to_owned()).ok()?,
                data: Some(value.to_owned()),
            })
        } else {
            Some(Self {
                method: FtpMethod::try_from(str.to_owned()).ok()?,
                data: None,
            })
        }
    }
}
impl Display for FtpPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.method.to_string())?;
        if let Some(data) = self.data.as_ref() {
            write!(f, " {}", data)?;
        }
        Ok(())
    }
}
enum FtpMethod {
    User,
    Pass,
    Acct,
    Cwd,
    Cdup,
    Smnt,
    Rein,
    Quit,
    Port,
    Pasv,
    Type,
    Stru,
    Mode,
    Retr,
    Stor,
    Stou,
    Appe,
    Allo,
    Rest,
    Rnfr,
    Abor,
    Dele,
    Rmd,
    Mkd,
    Pwd,
    List,
    Nlst,
    Site,
    Syst,
    Stat,
    Help,
    Noop,
}
impl ToString for FtpMethod {
    fn to_string(&self) -> String {
        match self {
            FtpMethod::User => "user",
            FtpMethod::Pass => "pass",
            FtpMethod::Acct => "acct",
            FtpMethod::Cwd => "cwd",
            FtpMethod::Cdup => "cdup",
            FtpMethod::Smnt => "smnt",
            FtpMethod::Rein => "rein",
            FtpMethod::Quit => "quit",
            FtpMethod::Port => "port",
            FtpMethod::Pasv => "pasv",
            FtpMethod::Type => "type",
            FtpMethod::Stru => "stru",
            FtpMethod::Mode => "mode",
            FtpMethod::Retr => "retr",
            FtpMethod::Stor => "stor",
            FtpMethod::Stou => "stou",
            FtpMethod::Appe => "appe",
            FtpMethod::Allo => "allo",
            FtpMethod::Rest => "rest",
            FtpMethod::Rnfr => "rnfr",
            FtpMethod::Abor => "abor",
            FtpMethod::Dele => "dele",
            FtpMethod::Rmd => "rmd",
            FtpMethod::Mkd => "mkd",
            FtpMethod::Pwd => "pwd",
            FtpMethod::List => "list",
            FtpMethod::Nlst => "nlst",
            FtpMethod::Site => "site",
            FtpMethod::Syst => "syst",
            FtpMethod::Stat => "stat",
            FtpMethod::Help => "help",
            FtpMethod::Noop => "noop",
        }
        .to_owned()
    }
}
impl TryFrom<String> for FtpMethod {
    type Error = ();
    fn try_from(value: String) -> Result<FtpMethod, ()> {
        Ok(match value.to_lowercase().as_str() {
            "user" => Self::User,
            "pass" => Self::Pass,
            "acct" => Self::Acct,
            "cwd" => Self::Cwd,
            "cdup" => Self::Cdup,
            "smnt" => Self::Smnt,
            "rein" => Self::Rein,
            "quit" => Self::Quit,
            "port" => Self::Port,
            "pasv" => Self::Pasv,
            "type" => Self::Type,
            "stru" => Self::Stru,
            "mode" => Self::Mode,
            "retr" => Self::Retr,
            "stor" => Self::Stor,
            "stou" => Self::Stou,
            "appe" => Self::Appe,
            "allo" => Self::Allo,
            "rest" => Self::Rest,
            "rnfr" => Self::Rnfr,
            "abor" => Self::Abor,
            "dele" => Self::Dele,
            "rmd" => Self::Rmd,
            "mkd" => Self::Mkd,
            "pwd" => Self::Pwd,
            "list" => Self::List,
            "nlst" => Self::Nlst,
            "site" => Self::Site,
            "syst" => Self::Syst,
            "stat" => Self::Stat,
            "help" => Self::Help,
            "noop" => Self::Noop,
            _ => return Err(()),
        })
    }
}
