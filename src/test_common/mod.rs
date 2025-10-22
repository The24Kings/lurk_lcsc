use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

static SERVER_CONN: OnceLock<TcpStream> = OnceLock::new();

pub(crate) fn setup() -> Arc<TcpStream> {
    // Start the server only once; I just need a valid TcpStream for testing.
    SERVER_CONN.get_or_init(|| {
        // Use a channel to share the listener's address with the main thread
        let (tx, rx) = std::sync::mpsc::channel();

        thread::spawn(move || {
            let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind TCP listener");
            let addr = listener.local_addr().unwrap();

            // Send the listener's address to the main thread
            tx.send(addr).expect("Failed to send address");

            if listener.accept().is_ok() {
                println!("Server accepted connection");
            } else {
                panic!("Failed to accept connection");
            }
        });

        // Give the server a moment to start
        thread::sleep(Duration::from_millis(100));

        // Receive the listener's address from the server thread
        let addr = rx.recv().expect("Failed to receive address");

        // Connect to the server using the received address
        TcpStream::connect(addr).expect("Failed to connect to server")
    });

    let stream = match SERVER_CONN.get() {
        Some(stream) => stream,
        None => panic!("Failed to get connection"),
    };

    Arc::new(stream.try_clone().expect("Failed to clone TcpStream"))
}
