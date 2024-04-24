use crate::{http::{
    http_methods::{delete, get, put},
    http_request::HttpRequest,
}, http_log};
use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    os::unix::ffi::OsStrExt,
    sync::Arc,
    thread::{self, sleep},
    time::Duration,
};

pub mod http_methods;
pub mod http_request;
static PATH: &str = "./static/users";
/// Creates a TcpListener on the provided address, accepting all incoming requests and sending the request to
/// ```no_run
/// handle_connection()
/// ```
/// to respond
/// # Errors
/// Returns an IO error if the TcpListener fails to bind to the requested address.
pub fn host_server(address: SocketAddr, max_threads: usize, salt: u128) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    let thread_count: Arc<()> = Arc::new(()); // Counts the number of threads spawned based on the weak count
    http_log!("==================== HTTP Server running on {address} ====================");
    for client in listener.incoming().flatten() {
        if Arc::strong_count(&thread_count) <= max_threads {
            /* Ignores request if too many threads are spawned */
            let passed_count = thread_count.clone();
            let new_addr = address.clone();
            if thread::Builder::new()
                .name("ClientHandler".to_string())
                .spawn(move || handle_connection(passed_count, client, new_addr, salt))
                .is_err()
            {
                /* Spawn thread to handle request */
                http_log!("Failed to spawn thread");
            }
        }
    }

    drop(thread_count);
    Ok(())
}
/// Takes in a threadcounter and TcpStream, reading the entire TCP packet before responding with the requested data. The `thread_counter` variable is dropped at the end of the function, such that the strong count represents the number of threads spawned.
fn handle_connection(thread_counter: Arc<()>, client: TcpStream, address: SocketAddr, salt: u128) {
    http_log!(
        "{} Thread(s) active.",
        Arc::strong_count(&thread_counter) - 1
    );
    let client_ip = client.peer_addr();
    client
        .set_read_timeout(Some(Duration::from_millis(100)))
        .expect("Should set read timeout");
    let mut packet = HttpRequest::new(client);
    if let Some(protocol) = packet.protocol() {
        match protocol.as_str() {
            "HTTP/1.1" | "undefined" => {
                if let Some(method) = packet.method() {
                    if let Ok(ip) = client_ip {
                        http_log!("Client {ip} made a {method} request");
                    } else {
                        http_log!("Client made a {method} request");
                    }
                    match method.to_lowercase().trim() {
                        "get" => get(packet, address, PATH),
                        "put" => {
                            put(packet, address, PATH, salt);
                        },
                        "delete" => {
                            delete(packet, PATH, salt);
                        },
                        _ => {
                            http_log!("Invalid method, request ignored.");
                            let _ = packet.respond_string("HTTP/1.1 405 Method Not Allowed\r\n\r\nUnknown request method. Allowed methods: \"GET\".\r\n");
                        }
                    }
                } else {
                    http_log!("No method provided");
                    let _ = packet.respond_string("HTTP/1.1 400 Bad Request\r\n\r\nUnknown request method. Allowed methods: \"GET\", \"PUT\", \"DELETE\".\r\n");
                }
            }
            proto => {
                http_log!("Client used invalid protocol: \"{proto}\"");
                let _ = packet.respond_string("Unknown protocol.");
            }
        }
    } else {
        http_log!("Client provided no protocol.");
    }

    drop(thread_counter); // Decrements the counter
}

fn garbage_collect(lifetime: Duration) {
    if let Ok(dir) = std::fs::read_dir(PATH) {
        for file in dir.flatten() {
            if let Ok(metadata) = file.metadata() {
                if let Ok(create_date) = metadata.created() {
                    if let Ok(elapsed) = create_date.elapsed() {
                        if elapsed > lifetime {
                            http_log!(
                                "Attempting garbage collection of \"{}\"",
                                String::from_utf8_lossy(file.file_name().as_bytes())
                            );
                            match std::fs::remove_dir_all(file.path()) {
                                Ok(()) => {
                                    http_log!(
                                        "Successfully deleted \"{}\"",
                                        String::from_utf8_lossy(file.file_name().as_bytes())
                                    );
                                }
                                Err(err) => {
                                    http_log!(
                                        "Failed to delete \"{}\": {}",
                                        String::from_utf8_lossy(file.file_name().as_bytes()),
                                        err
                                    );
                                }
                            }
                        }
                    } else {
                        http_log!(
                            "Failed to get time since creation of \"{}\"",
                            String::from_utf8_lossy(file.file_name().as_bytes())
                        )
                    }
                } else {
                    http_log!(
                        "Failed to get creation date of \"{}\"",
                        String::from_utf8_lossy(file.file_name().as_bytes())
                    )
                }
            } else {
                http_log!(
                    "Failed to get metadata of \"{}\"",
                    String::from_utf8_lossy(file.file_name().as_bytes())
                );
            }
        }
    }
}
fn garbage_collector_loop(lifetime: Duration) {
    thread::Builder::new()
        .name("Garbage collector".to_owned())
        .spawn(move || loop {
            garbage_collect(lifetime);
            sleep(Duration::from_secs(60 * 60))
        })
        .expect("Failed to spawn garbage collector");
}

#[macro_export]
macro_rules! http_log {
    () => {
        use std::io::Write;
        let current_time: DateTime<Utc> = Utc::now();
        std::fs::OpenOptions::new().append(true).open("logs/http.log").expect("Failed to open http.log file").write_all(format!("[{} UTC] {}:{}:{}\n", current_time.format("%Y-%m-%d %H:%M:%S"), file!(), line!(), column!()).as_bytes()).expect("Failed to write to log file");
        println!("[{} UTC] {}:{}:{}", current_time.format("%Y-%m-%d %H:%M:%S"), file!(), line!(), column!());
    };
    ($($arg:tt)*) => {{
        use std::io::Write;
        let current_time: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
        std::fs::OpenOptions::new().append(true).open("logs/http.log").expect("Failed to open http.log file").write_all(format!("[{} UTC] {}:{}:{}: {}\n", current_time.format("%Y-%m-%d %H:%M:%S"), file!(), line!(), column!(), format!($($arg)*)).as_bytes()).expect("Failed to write to log file");
        println!("[{} UTC] {}:{}:{}: {}", current_time.format("%Y-%m-%d %H:%M:%S"), file!(), line!(), column!(), format!($($arg)*));
    }};
}
