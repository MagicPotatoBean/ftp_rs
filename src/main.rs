use std::{net::{TcpListener, SocketAddrV4, Ipv4Addr, TcpStream}, io::{Read, Write}, time::Duration};
fn main() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 65432)).expect("Failed to bind to port.");
    for client in listener.incoming() {
        println!("Got tcp connection.");
        match client {
            Ok(mut stream) => {
                stream.write_all(FtpResponseCode::Greeting.to_string().as_bytes()).unwrap();
                let mut username = {
                    let raw = read_until_consume(&stream, '\r');
                    stream.read(&mut [0u8]).unwrap();
                    String::from_utf8_lossy(&raw).to_string().split_off(5)
                };
                println!("Username : \"{}\"", &username);


                stream.write_all(FtpResponseCode::UsernameOkNeedPassword.to_string().as_bytes()).unwrap();
                let mut password = {
                    let raw = read_until_consume(&stream, '\r');
                    stream.read(&mut [0u8]).unwrap();
                    String::from_utf8_lossy(&raw).to_string().split_off(5)
                };
                println!("Password : \"{}\"", &password);

                stream.write_all(FtpResponseCode::UsernameOkNeedPassword.to_string().as_bytes()).unwrap();
                let mut request = {
                    let raw = read_until_consume(&stream, '\r');
                    stream.read(&mut [0u8]).unwrap();
                    String::from_utf8_lossy(&raw).to_string().split_off(5)
                };
                println!("Account : \"{}\"", &request);



                read_to_end(&mut stream);
            },
            Err(_) => println!("Failed to connect."),
        }

    }
}
fn read_to_end(stream: &mut TcpStream) {
    let mut vec = Vec::new();
    stream.set_nonblocking(true);
    stream.read_to_end(&mut vec);
    println!("Remaining data; \"{}\"", String::from_utf8_lossy(&vec));
}
fn read_until_consume(mut stream: &TcpStream, chr: char) -> Vec<u8> {
    let mut byte = [0u8];
    let mut output = Vec::new();
    let delay = stream.read_timeout().unwrap();
    loop {
        match stream.read(&mut byte).map_err(|err| err.kind()) {
            Ok(_) => {
                if char::from(byte[0]) == chr {
                    break output;
                }
                output.push(byte[0]);
            },
            Err(err) => {
                println!("Encountered an error: {err}");
            }
        }
        
    }
}
enum FtpResponseCode {
    Greeting,
    UsernameOkNeedPassword,
    LoggedInProceed,
}
impl ToString for FtpResponseCode {
    fn to_string(&self) -> String {
        match self {
            FtpResponseCode::Greeting => "220\n",
            FtpResponseCode::UsernameOkNeedPassword => "331\n",
            FtpResponseCode::LoggedInProceed => "230\n",
        }.to_owned()
    }
}
struct FtpPacket {
    method: FtpMethod,
    data: String,
}
impl FtpPacket {
    fn new(str: String) -> Option<Self> {
        let (method, value) = str.split_once(" ")?;
        Some(Self {
            method: FtpMethod::try_from(method.to_owned()).ok()?,
            data: value.to_owned()
        })
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
    Stat,
    Help,
    Noop,
}
impl ToString for FtpMethod {
    fn to_string(&self) -> String {
        match self {
            FtpMethod::User => "User",
            FtpMethod::Pass => "Pass",
            FtpMethod::Acct => "Acct",
            FtpMethod::Cwd => "Cwd",
            FtpMethod::Cdup => "Cdup",
            FtpMethod::Smnt => "Smnt",
            FtpMethod::Rein => "Rein",
            FtpMethod::Quit => "Quit",
            FtpMethod::Port => "Port",
            FtpMethod::Pasv => "Pasv",
            FtpMethod::Type => "Type",
            FtpMethod::Stru => "Stru",
            FtpMethod::Mode => "Mode",
            FtpMethod::Retr => "Retr",
            FtpMethod::Stor => "Stor",
            FtpMethod::Stou => "Stou",
            FtpMethod::Appe => "Appe",
            FtpMethod::Allo => "Allo",
            FtpMethod::Rest => "Rest",
            FtpMethod::Rnfr => "Rnfr",
            FtpMethod::Abor => "Abor",
            FtpMethod::Dele => "Dele",
            FtpMethod::Rmd => "Rmd",
            FtpMethod::Mkd => "Mkd",
            FtpMethod::Pwd => "Pwd",
            FtpMethod::List => "List",
            FtpMethod::Nlst => "Nlst",
            FtpMethod::Site => "Site",
            FtpMethod::Stat => "Stat",
            FtpMethod::Help => "Help",
            FtpMethod::Noop => "Noop",
        }.to_owned()
    }
}
impl TryFrom<String> for FtpMethod {
    type Error = ();
    fn try_from(value: String) -> Result<FtpMethod, ()> {
        Ok(match value.as_str() {
            "User" => {Self::User}
"Pass" => {Self::Pass}
"Acct" => {Self::Acct}
"Cwd" => {Self::Cwd}
"Cdup" => {Self::Cdup}
"Smnt" => {Self::Smnt}
"Rein" => {Self::Rein}
"Quit" => {Self::Quit}
"Port" => {Self::Port}
"Pasv" => {Self::Pasv}
"Type" => {Self::Type}
"Stru" => {Self::Stru}
"Mode" => {Self::Mode}
"Retr" => {Self::Retr}
"Stor" => {Self::Stor}
"Stou" => {Self::Stou}
"Appe" => {Self::Appe}
"Allo" => {Self::Allo}
"Rest" => {Self::Rest}
"Rnfr" => {Self::Rnfr}
"Abor" => {Self::Abor}
"Dele" => {Self::Dele}
"Rmd" => {Self::Rmd}
"Mkd" => {Self::Mkd}
"Pwd" => {Self::Pwd}
"List" => {Self::List}
"Nlst" => {Self::Nlst}
"Site" => {Self::Site}
"Stat" => {Self::Stat}
"Help" => {Self::Help}
"Noop" => {Self::Noop}
_ => return Err(())
        })
    }
}