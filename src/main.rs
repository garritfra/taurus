extern crate native_tls;
extern crate url;

mod gemini;

use native_tls::{Identity, TlsAcceptor, TlsStream};
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path;
use std::sync::Arc;
use std::thread;

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
                    Ok(stream) => {
                        handle_client(stream).unwrap_or_else(|e| println!("Error: {}", e))
                    }
                    Err(e) => println!("Can't handle stream: {}", e),
                });
            }
            Err(err) => println!("Error: {}", err),
        }
    }
}

/// Helper function to read a file into Vec
fn read_file(file_path: &str) -> Result<Vec<u8>, io::Error> {
    let mut file = File::open(file_path)?;
    let mut buf = Vec::new();

    file.read_to_end(&mut buf)?;

    Ok(buf)
}

/// Send file as a response
fn send_file(path: &str, response: &mut gemini::GeminiResonse) {
    match read_file(path) {
        Ok(buf) => {
            response.body = Some(buf);
        }
        Err(err) => {
            // Cannot read file or it doesn't exist

            println!("Error ({}): {}", path, err);

            response.status = [b'5', b'1'];
            response.meta = format!("Resource not found: {}", path).into();
        }
    }
}

fn not_found(path: &str, response: &mut gemini::GeminiResonse) {
    response.status = [b'5', b'1'];
    response.meta = format!("Resource not found: {}", path).into();
}

fn redirect(path: &str, response: &mut gemini::GeminiResonse) {
    response.status = [b'3', b'1'];
    response.meta = path.into();
}

fn handle_client(mut stream: TlsStream<TcpStream>) -> Result<(), String> {
    let mut buffer = [0; 1024];
    if let Err(e) = stream.read(&mut buffer) {
        println!("Could not read from stream: {}", e)
    }

    let mut raw_request = String::from_utf8_lossy(&buffer[..]).to_mut().to_owned();

    // TODO: Redundantly converted to owned and later referenced again
    if !raw_request.starts_with("gemini://") {
        raw_request = "gemini://".to_owned() + &raw_request;
    }

    let request = gemini::GeminiRequest::from_string(&raw_request).unwrap();
    let mut response = gemini::GeminiResonse::new();

    let url_path = request.file_path();
    let file_path = path::Path::new(url_path);

    if file_path.has_root() {
        // File starts with `/` (*nix) or `\\` (Windows), decline it
        not_found(url_path, &mut response);
    } else {
        let path = path::Path::new(".").join(file_path).as_path().to_owned();

        // Check if file/dir exists
        if path.exists() {
            // If it's a directory, try to find index.gmi
            if path.is_dir() {
                redirect(
                    path.join("index.gmi")
                        .iter()
                        .skip(1)
                        .collect::<path::PathBuf>()
                        .to_str()
                        .ok_or("Invalid Unicode".to_owned())?,
                    &mut response,
                );
            } else {
                send_file(
                    path.to_str().ok_or("Invalid Unicode".to_owned())?,
                    &mut response,
                );
            }
        } else {
            not_found(url_path, &mut response);
        }
    }

    if let Err(e) = stream.write(&response.build()) {
        println!("Could not write to stream: {}", e);
    }

    Ok(())
}
