use std::io::Read;
use std::io::Write;
use std::net::TcpListener;

fn main() {
    // 1965 is the standard port for gemini
    let port = "1965";
    let address = format!("0.0.0.0:{}", port);
    let listener =
        TcpListener::bind(address).unwrap_or_else(|_| panic!("Could not bind to port {}", port));

    println!("Listening on port 1965");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buffer = [0; 1024];
            if let Err(e) = stream.read(&mut buffer) {
                println!("Could not read from stream: {}", e)
            }

            if let Err(e) = stream.write(b"HELLO") {
                println!("Could not write to stream: {}", e);
            }
        }
    }
}
