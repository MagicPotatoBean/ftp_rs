use std::{fmt::Display, net::TcpStream, io::Write, path::{PathBuf, Component}};

use crate::FtpState;

pub enum FtpCode {
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
impl FtpCode {
    pub fn send(&self, stream: &mut TcpStream, msg: &str) -> Result<(), std::io::Error> {
        stream.write_all(self.to_string(msg).as_bytes())
    }
    pub fn to_string(&self, msg: &str) -> String {
        let mut code = match self {
            FtpCode::CmdOk => 200,
            FtpCode::CmdSyntaxErr => 500,
            FtpCode::ParamSyntaxErr => 501,
            FtpCode::CmdNotNeeded => 202,
            FtpCode::CmdNotImpl => 502,
            FtpCode::BadSequence => 503,
            FtpCode::CmdNotImplForParam => 504,
            FtpCode::RestartMarker => 110,
            FtpCode::SystStatusOrHelp => 211,
            FtpCode::DirStatus => 212,
            FtpCode::FileStatus => 213,
            FtpCode::HelpMsg => 214,
            FtpCode::SystemName => 215,
            FtpCode::ReadyInNMinutes => 120,
            FtpCode::ReadyForNewUser => 220,
            FtpCode::ServiceClosing => 221,
            FtpCode::ServiceNotAvailable => 421,
            FtpCode::DataConOpenTransferStarting => 125,
            FtpCode::DataConOpenNoTransfer => 225,
            FtpCode::CantOpenDataCon => 425,
            FtpCode::ConClosedRequestSuccess => 226,
            FtpCode::ConClosedRequestAborted => 426,
            FtpCode::EnteringPassive => 227,
            FtpCode::LoggedInProceed => 230,
            FtpCode::NotLoggedIn => 530,
            FtpCode::UnOkNeedPw => 331,
            FtpCode::NeedAcctForLogin => 332,
            FtpCode::NeedAcctForFiles => 532,
            FtpCode::FileOkOpeningDataCon => 150,
            FtpCode::RequestCompleted => 250,
            FtpCode::FileCreated => 257,
            FtpCode::RequestedFileActionRequiresInfo => 350,
            FtpCode::FileBusy => 450,
            FtpCode::FileNotFoundOrInvalidPerms => 550,
            FtpCode::RequestAbortedLocalErr => 451,
            FtpCode::RequestAbortedPageErr => 551,
            FtpCode::InsufficientStorage => 452,
            FtpCode::ExceededStorageAllocation => 552,
            FtpCode::FileNameDisallowed => 553,
        }
        .to_string();
        if msg.len() >= 1 {
            code.push(' ');
            code.push_str(msg);
        }
        code.push('\n');
        code
    }
    pub fn from_string(string: &str) -> Option<Self> {
        Some(match string {
            "200" => FtpCode::CmdOk,
            "500" => FtpCode::CmdSyntaxErr,
            "501" => FtpCode::ParamSyntaxErr,
            "202" => FtpCode::CmdNotNeeded,
            "502" => FtpCode::CmdNotImpl,
            "503" => FtpCode::BadSequence,
            "504" => FtpCode::CmdNotImplForParam,
            "110" => FtpCode::RestartMarker,
            "211" => FtpCode::SystStatusOrHelp,
            "212" => FtpCode::DirStatus,
            "213" => FtpCode::FileStatus,
            "214" => FtpCode::HelpMsg,
            "215" => FtpCode::SystemName,
            "120" => FtpCode::ReadyInNMinutes,
            "220" => FtpCode::ReadyForNewUser,
            "221" => FtpCode::ServiceClosing,
            "421" => FtpCode::ServiceNotAvailable,
            "125" => FtpCode::DataConOpenTransferStarting,
            "225" => FtpCode::DataConOpenNoTransfer,
            "425" => FtpCode::CantOpenDataCon,
            "226" => FtpCode::ConClosedRequestSuccess,
            "426" => FtpCode::ConClosedRequestAborted,
            "227" => FtpCode::EnteringPassive,
            "230" => FtpCode::LoggedInProceed,
            "530" => FtpCode::NotLoggedIn,
            "331" => FtpCode::UnOkNeedPw,
            "332" => FtpCode::NeedAcctForLogin,
            "532" => FtpCode::NeedAcctForFiles,
            "150" => FtpCode::FileOkOpeningDataCon,
            "250" => FtpCode::RequestCompleted,
            "257" => FtpCode::FileCreated,
            "350" => FtpCode::RequestedFileActionRequiresInfo,
            "450" => FtpCode::FileBusy,
            "550" => FtpCode::FileNotFoundOrInvalidPerms,
            "451" => FtpCode::RequestAbortedLocalErr,
            "551" => FtpCode::RequestAbortedPageErr,
            "452" => FtpCode::InsufficientStorage,
            "552" => FtpCode::ExceededStorageAllocation,
            "553" => FtpCode::FileNameDisallowed,
            _ => return None,
        })
    }
}
pub struct FtpPacket {
    pub method: FtpMethod,
    pub data: Option<String>,
}
impl FtpPacket {
    pub fn new(str: String) -> Option<Self> {
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
pub enum FtpMethod {
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
pub fn is_owned(permission_dir: &PathBuf, path: &PathBuf) -> bool {
    match path.canonicalize() {
        Ok(path) => {
            path.starts_with(permission_dir)
        },
        Err(err) => {
            println!("Error in is_owned : {}", err.kind());
            path.starts_with(permission_dir) && path.components().all(|comp| comp != Component::ParentDir)
        },
    }
}