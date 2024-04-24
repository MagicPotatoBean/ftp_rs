use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io::{self, Read, Write},
    net::SocketAddr,
    path::{Component, PathBuf},
};

use crate::http::{http_log, http_request::HttpRequest};
/// Hashes the current system time, converts it to hex, makes a file with that name and stores the packet body to that file
pub fn put(mut packet: HttpRequest, address: SocketAddr, path: &'static str, salt: u128) {
    if let Some(Ok(creds)) = packet
        .headers()
        .unwrap()
        .get("Authorization")
        .map(|item| http_auth_basic::Credentials::from_header(item.to_owned()))
    {
        if creds.user_id.chars().all(|chr| chr.is_ascii_alphanumeric()) {
            if let Ok(auth_data) =
                std::fs::read(PathBuf::from(path).join(&&creds.user_id).join("auth"))
            {
                let mut hasher = DefaultHasher::new();
                creds.user_id.hash(&mut hasher);
                creds.password.hash(&mut hasher);
                salt.hash(&mut hasher);
                let pass_hash = hasher.finish();
                if pass_hash.to_be_bytes().to_vec() == auth_data {
                    http_log!("Authenticated");
                } else {
                    return;
                }
            } else {
                let mut hasher = DefaultHasher::new();
                creds.user_id.hash(&mut hasher);
                creds.password.hash(&mut hasher);
                salt.hash(&mut hasher);
                let pass_hash = hasher.finish();
                if std::fs::write(
                    PathBuf::from(path).join(&creds.user_id).join("auth"),
                    pass_hash.to_be_bytes(),
                )
                .is_err()
                {
                    return;
                }
            }

            http_log!("username: {}, password: {}", creds.user_id, creds.password);
            if let Some(name) = packet.path() {
                let name = &name[1..]; // Remove leading "/"
                let mut is_100_continue = false;
                if let Some(headers) = packet.headers() {
                    for (header, value) in headers {
                        if header == "Expect" && value == "100-continue" {
                            is_100_continue = true;
                        }
                    }
                }
                if is_100_continue
                    && packet
                        .respond_string("HTTP/1.1 100 Continue\r\n\r\n")
                        .is_err()
                {
                    http_log!("Failed to 100-continue");
                }

                let file_location = PathBuf::from(path)
                    .join(&creds.user_id)
                    .join("files")
                    .join(&name); // Make sure the path doesnt include .. for path traversal
                http_log!("file_location:{}", file_location.display());
                if !(file_location
                    .components()
                    .all(|comp| comp != Component::ParentDir)
                    && file_location
                        .starts_with(&PathBuf::from(&path).join(&creds.user_id).join("files")))
                {
                    http_log!(
                        "Request rejected: \"{}\" doesnt start with \"{}\"",
                        file_location.display(),
                        PathBuf::from(&path)
                            .join(&creds.user_id)
                            .join("files")
                            .display()
                    );
                    packet.respond_string("HTTP/1.1 403 Forbidden\r\n\r\nYou do not have permission to access this directory\r\n").unwrap();
                } else {
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(&file_location)
                    {
                        // Read byte-by-byte from client and send to file.
                        loop {
                            let mut byte = [0u8];
                            match packet.body_stream().read(&mut byte) {
                                Ok(_) => {
                                    if file.write(&byte).is_err() {
                                        http_log!(
                                            "Failed to write byte to file \"{}\"",
                                            file_location.display()
                                        );
                                    }
                                }
                                Err(err) => match err.kind() {
                                    io::ErrorKind::Interrupted => http_log!("Interrupted"),
                                    io::ErrorKind::WouldBlock => break,
                                    err => {
                                        http_log!("Stopped writing to file: \"{err}\"");
                                        break;
                                    }
                                },
                            }
                        }
                        let mut addr = address.to_string();
                        if let Some(header_map) = packet.headers() {
                            if let Some(host_addr) = header_map.get("Host") {
                                addr = host_addr.to_owned();
                                addr.push_str(":");
                                addr.push_str(&address.port().to_string());
                            }
                        }
                        if packet
                            .respond_string(&format!(
                                "HTTP/1.1 200 Ok\r\n\r\nhttp://{}/{}\r\n",
                                addr,
                                PathBuf::from(creds.user_id)
                                    .join(name)
                                    .display()
                                    .to_string()
                            ))
                            .is_err()
                        {
                            http_log!(
                                "Failed to send user path to access file \"{}\"",
                                file_location.display()
                            );
                        }
                    } else {
                        http_log!("Failed to create file \"{}\"", file_location.display());
                    }
                }
            }
        }
    } else {
        if packet.respond_string(&format!("HTTP/1.1 401 Unauthorized\r\nWWW-Authenticate: Basic\r\n")).is_err() {
            http_log!("Failed to send user packet requiring authentication");
        }
    }
    packet.read_all();
    http_log!("{packet}\n");
}
pub fn web_page(page: &str, packet: &mut HttpRequest, address: SocketAddr) {
    match page {
        "" => {
            let no_html;
            if let Some(headers) = packet.headers() {
                if let Some(user_agent) = headers.get("Accept") {
                    no_html = !user_agent.contains("text/html");
                } else {
                    no_html = true; // Assumes this is a basic custom TUI
                }
            } else {
                no_html = true; // Assumes this is a basic custom TUI
            }
            if no_html {
                let mut addr = address.to_string();
                if let Some(header_map) = packet.headers() {
                    if let Some(host_addr) = header_map.get("Host") {
                        addr = host_addr.to_owned();
                        addr.push_str(":");
                        addr.push_str(&address.port().to_string());
                    }
                }
                let _ = packet.respond_string( &format!("HTTP/1.1 200 Ok\r\n\r\nTo upload, type:\r\n$ curl --upload-file <filename> http://{addr} -u username:password\r\n\r\nTo download, type:\r\n$ curl http://{addr}/<username>/<file_name> --output filename.txt\r\n(account not required)\r\n\r\nAnd to delete, type: \r\n$ curl -X DELETE http://{addr}/<username>/<file_name> -u username:password\r\n\r\nIf you would like this output to be in HTML, please add \"text/html\" as an accepted format in your \"Accept\" header."));
            } else {
                let _ = packet.respond_string("HTTP/1.1 200 OK\r\n\r\n");
                let _ = packet
                    .respond_data(&std::fs::read("site/index.html").expect("Missing html page."));
            }
        }
        "styles.css" => {
            let _ = packet.respond_string("HTTP/1.1 200 OK\r\n\r\n");
            let _ =
                packet.respond_data(&std::fs::read("site/styles.css").expect("Missing css page."));
        }
        "script.js" => {
            let _ = packet.respond_string("HTTP/1.1 200 OK\r\n\r\n");
            let _ =
                packet.respond_data(&std::fs::read("site/script.js").expect("Missing js page."));
        }
        "favicon.ico" => {
            let _ = packet.respond_string("HTTP/1.1 200 OK\r\n\r\n");
            let _ =
                packet.respond_data(&std::fs::read("site/favicon.ico").expect("Missing ico page."));
        }
        _ => {
            http_log!("Unconfigured main page file requested: {page}")
        }
    }
}
// Reads the requested path, and if it matches a file on the server, returns the file in the body
pub fn get(mut packet: HttpRequest, address: SocketAddr, path: &'static str) {
    if let Some(name) = packet.path() {
        let name = &name[1..];
        if name == "" || name == "styles.css" || name == "script.js" || name == "favicon.ico" {
            http_log!("Requesting main page \"{name}\"");
            web_page(name, &mut packet, address);
        } else {
            if let Some((username, file_path)) = name.split_once("/") {
                if let Ok(file_location) = PathBuf::from(path)
                    .join(username)
                    .join("files")
                    .join(file_path)
                    .canonicalize()
                {
                    if let Ok(root_dir) = PathBuf::from(path)
                        .join(username)
                        .join("files")
                        .canonicalize()
                    {
                        if file_location.starts_with(root_dir)
                            && username.chars().all(|chr| chr.is_ascii_alphanumeric())
                        {
                            if let Ok(mut file) =
                                std::fs::OpenOptions::new().read(true).open(&file_location)
                            {
                                let _ = packet.respond_string("HTTP/1.1 200 Ok\r\n\r\n"); // Send header so client is ready to receive file
                                                                                          // Read file byte-by-byte, sending each byte to the client.
                                loop {
                                    let mut byte = [0u8];
                                    match file.read(&mut byte) {
                                        Ok(num) => {
                                            if num == 0 {
                                                break;
                                            }
                                            packet.respond_data(&byte).unwrap();
                                        }
                                        Err(err) => match err.kind() {
                                            io::ErrorKind::UnexpectedEof
                                            | io::ErrorKind::Interrupted => {}
                                            _ => break, // When reached end of file, break.
                                        },
                                    }
                                }
                            }
                        } else {
                            packet
                                .respond_string(&format!(
                                    "HTTP/1.1 404 Not found\r\n\r\n404 File not found\r\n"
                                ))
                                .unwrap();
                            http_log!(
                                "Client requested non-existent file \"{}\"",
                                file_location.display()
                            );
                        }
                    } else {
                        packet
                            .respond_string(&format!(
                                "HTTP/1.1 404 Not found\r\n\r\n404 File not found\r\n"
                            ))
                            .unwrap();
                        http_log!(
                            "Client requested non-existent file \"{}\"",
                            file_location.display()
                        );
                    }
                } else {
                    packet
                        .respond_string(&format!(
                            "HTTP/1.1 404 Not found\r\n\r\n404 File not found\r\n"
                        ))
                        .unwrap();
                    http_log!(
                        "Client requested non-existent file \"{}\"",
                        PathBuf::from(path)
                            .join(username)
                            .join("files")
                            .join(file_path)
                            .display()
                    );
                }
            }
        }
    }
    packet.read_all();
    http_log!("{packet}\n");
}
pub fn delete(mut packet: HttpRequest, path: &'static str, salt: u128) {
    if let Some(Ok(creds)) = packet
        .headers()
        .unwrap()
        .get("Authorization")
        .map(|item| http_auth_basic::Credentials::from_header(item.to_owned()))
    {
        if creds.user_id.chars().all(|chr| chr.is_ascii_alphanumeric()) {
            if let Ok(auth_data) =
                std::fs::read(PathBuf::from(path).join(&&creds.user_id).join("auth"))
            {
                let mut hasher = DefaultHasher::new();
                creds.user_id.hash(&mut hasher);
                creds.password.hash(&mut hasher);
                salt.hash(&mut hasher);
                let pass_hash = hasher.finish();
                if pass_hash.to_be_bytes().to_vec() == auth_data {
                    http_log!("Authenticated");
                } else {
                    return;
                }
            } else {
                let mut hasher = DefaultHasher::new();
                creds.user_id.hash(&mut hasher);
                creds.password.hash(&mut hasher);
                salt.hash(&mut hasher);
                let pass_hash = hasher.finish();
                if std::fs::write(
                    PathBuf::from(path).join(&creds.user_id).join("auth"),
                    pass_hash.to_be_bytes(),
                )
                .is_err()
                {
                    return;
                }
            }
            if let Some(name) = packet.path() {
                let name = &name[1..];
                if let Ok(file_location) = PathBuf::from(path)
                    .join(&creds.user_id)
                    .join("files")
                    .join(&name)
                    .canonicalize()
                {
                    if file_location.starts_with(PathBuf::from(path)
                    .join(&creds.user_id)
                    .join("files")) {
                        if std::fs::remove_file(file_location).is_err() {
                            if packet.respond_string("HTTP/1.1 500 Failed to delete file.").is_err() {
                                http_log!("Failed to send user error message");
                            }
                        }
                    }
                } else {
                }
            }
        }
    }

    packet.read_all();
    http_log!("{packet}\n");
}
