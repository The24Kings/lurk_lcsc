use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, OnceLock},
    thread,
    time::Duration,
};

static SERVER_ADDR: OnceLock<SocketAddr> = OnceLock::new();

pub fn setup() -> Arc<TcpStream> {
    // Start the server only once
    SERVER_ADDR.get_or_init(|| {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind TCP listener");
        let addr = listener.local_addr().expect("Failed to get local address");

        // Listen for incoming connections
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(_stream) => {
                        println!("Server accepted connection");
                        break; // Accept one connection and exit
                    }
                    Err(e) => {
                        eprintln!("Connection failed: {}", e);
                        break;
                    }
                }
            }
        });

        // Give the server a moment to start
        thread::sleep(Duration::from_millis(100));

        addr
    });

    let addr = match SERVER_ADDR.get() {
        Some(addr) => addr,
        None => panic!("Failed to get connection"),
    };

    // Connect to the server using the stored address
    let stream = TcpStream::connect(addr).expect("Failed to connect to server");

    Arc::new(stream)
}
