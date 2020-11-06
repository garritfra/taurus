extern crate native_tls;
extern crate url;

use native_tls::{Identity, TlsAcceptor, TlsStream};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use url::Url;

fn main() {
    let mut file =
        File::open("identity.pfx").expect("File identity.pfx not found in current directory");
    let mut identity = vec![];
    file.read_to_end(&mut identity)
        .expect("Cannot read identity.pfx");
    let identity = Identity::from_pkcs12(&identity, "qqqq").unwrap();

    // 1965 is the standard port for gemini
    let port = "1965";
    let address = format!("0.0.0.0:{}", port);
    let listener =
        TcpListener::bind(address).unwrap_or_else(|_| panic!("Could not bind to port {}", port));
    let acceptor = TlsAcceptor::new(identity).unwrap();
    let acceptor = Arc::new(acceptor);

    println!("Listening on port 1965");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let acceptor = acceptor.clone();
                thread::spawn(move || match acceptor.accept(stream) {
                    Ok(stream) => handle_client(stream),
                    Err(e) => println!("Can't handle stream: {}", e),
                });
            }
            Err(_e) => println!("Error: {}", _e),
        }
    }
}

fn handle_client(mut stream: TlsStream<TcpStream>) {
    let mut buffer = [0; 1024];
    if let Err(e) = stream.read(&mut buffer) {
        println!("Could not read from stream: {}", e)
    }

    let mut raw_request = String::from_utf8_lossy(&buffer[..]).to_mut().to_owned();

    if !raw_request.starts_with("gemini://") {
        raw_request = "gemini://".to_owned() + &raw_request;
    }

    let request = Url::parse(&raw_request).expect("Can not parse URL");
    let mut response: Vec<u8> = Vec::new();

    // 20 SUCESS status
    response.extend("20".as_bytes());

    // <Space>
    response.push(0x20);

    // <Meta>
    response.extend("text/gemini".as_bytes());

    response.extend("\r\n".as_bytes());

    let file_path = request
        .path()
        .chars()
        .next()
        .map(|c| &request.path()[c.len_utf8()..])
        .unwrap();

    println!("Reading {}", file_path);

    let mut file = File::open(file_path).unwrap();
    if let Err(e) = file.read_to_end(&mut response) {
        println!("Could not read file {}", e);
    }

    if let Err(e) = stream.write(&response) {
        println!("Could not write to stream: {}", e);
    }
}
