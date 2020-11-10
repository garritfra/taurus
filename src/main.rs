extern crate native_tls;
extern crate url;

mod config;
mod error;
mod gemini;

use native_tls::{Identity, TlsAcceptor, TlsStream};
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path;
use std::sync::Arc;
use std::thread;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

fn main() -> Result<(), error::SimpleError> {
    // CLI
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .long("config")
                .short("c")
                .help("Alternative config file location [default /etc/taurus/taurus.toml]")
                .next_line_help(true)
                .value_name("FILE"),
        )
        .get_matches();

    let config_path = matches.value_of("config").map(|v| v.to_owned());
    let config: config::Config = config::Config::load(config_path)
        .map_err(|err| format!("failed to read configuration file: {}", err))?;

    // Defaults for configuration file
    let port = config.port.unwrap_or(1965);
    let cert_file = config
        .certificate_file
        .unwrap_or_else(|| "/etc/taurus/identity.pfx".to_owned());
    let static_root = config
        .static_root
        .unwrap_or_else(|| "/var/www/gemini".to_owned());

    // Read certificate
    let mut file =
        File::open(cert_file).map_err(|err| format!("failed to open identity file: {}", err))?;

    let mut identity = vec![];
    file.read_to_end(&mut identity)
        .map_err(|err| format!("failed to read identity file: {}", err))?;

    let identity = Identity::from_pkcs12(&identity, &config.certificate_password)
        .map_err(|err| format!("failed to parse certificate: {}", err))?;

    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(address).map_err(|err| format!("failed to bind: {}", err))?;
    let acceptor = TlsAcceptor::new(identity).unwrap();
    let acceptor = Arc::new(acceptor);

    println!("Info: Listening on port {}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let acceptor = acceptor.clone();
                let static_root = static_root.clone();

                thread::spawn(move || match acceptor.accept(stream) {
                    Ok(stream) => handle_client(stream, &static_root)
                        .unwrap_or_else(|e| println!("Error: {}", e)),
                    Err(e) => println!("Error: can't handle stream: {}", e),
                });
            }
            Err(err) => println!("Error: {}", err),
        }
    }

    Ok(())
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

            println!("Error [{}]: {}", path, err);

            response.status = [b'5', b'1'];
            response.meta = format!("Resource not found: {}", path).into();
        }
    }
}

fn not_found(path: &str, response: &mut gemini::GeminiResonse) {
    response.status = [b'5', b'1'];
    response.meta = format!("Resource not found: {}", path).into();
}

fn handle_client(mut stream: TlsStream<TcpStream>, static_root: &str) -> Result<(), String> {
    let mut buffer = [0; 1024];
    if let Err(e) = stream.read(&mut buffer) {
        return Err(format!("could not read from stream: {}", e));
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

        let actual_path = path::Path::new(&static_root)
            .join(&path)
            .as_path()
            .to_owned();

        // Check if file/dir exists
        if actual_path.exists() {
            // If it's a directory, try to find index.gmi
            if actual_path.is_dir() {
                let index_path = actual_path
                    .join("index.gmi")
                    .to_str()
                    .ok_or("invalid Unicode".to_owned())?
                    .to_owned();

                send_file(&index_path, &mut response);
            } else {
                send_file(
                    actual_path.to_str().ok_or("invalid Unicode".to_owned())?,
                    &mut response,
                );
            }
        } else {
            not_found(url_path, &mut response);
        }
    }

    if let Err(e) = stream.write(&response.build()) {
        return Err(format!("could not write to stream: {}", e));
    }

    Ok(())
}
