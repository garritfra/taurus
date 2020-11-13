extern crate native_tls;
extern crate url;

mod config;
mod error;
mod gemini;
mod logger;

use error::{TaurusError, TaurusResult};
use gemini::{GeminiRequest, GeminiResponse};
use native_tls::{Identity, TlsAcceptor, TlsStream};
use std::{
    fs::File,
    io::{self, Read},
    net::{TcpListener, TcpStream},
    path,
    sync::Arc,
    thread,
};

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

fn main() {
    if let Err(e) = run() {
        logger::error(e);
        std::process::exit(1);
    }
}

fn run() -> TaurusResult<()> {
    // CLI
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .long("config")
                .short("c")
                .help("Specify an alternative config file")
                .default_value("/etc/taurus/taurus.toml")
                .next_line_help(true)
                .value_name("FILE"),
        )
        .get_matches();

    let config_path = matches.value_of("config").unwrap();
    let config: config::Config =
        config::Config::load(config_path).map_err(TaurusError::InvalidConfig)?;

    // Defaults for configuration file
    let port = config.port.unwrap_or(1965);
    let cert_file = config
        .certificate_file
        .unwrap_or_else(|| "/etc/taurus/identity.pfx".to_owned());
    let static_root = config
        .static_root
        .unwrap_or_else(|| "/var/www/gemini".to_owned());

    // Read certificate
    let identity = read_file(&cert_file).map_err(TaurusError::NoIdentity)?;

    let identity = Identity::from_pkcs12(&identity, &config.certificate_password)?;

    let address = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(address).map_err(TaurusError::BindFailed)?;
    let acceptor = TlsAcceptor::new(identity).unwrap();
    let acceptor = Arc::new(acceptor);

    logger::info(format!("Listening on port {}", port));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let acceptor = acceptor.clone();
                let static_root = static_root.clone();

                thread::spawn(move || match acceptor.accept(stream) {
                    Ok(stream) => {
                        if let Err(e) = handle_client(stream, &static_root) {
                            logger::error(format!("Can't handle client: {}", e));
                        }
                    }
                    Err(e) => {
                        logger::error(format!("Can't handle stream: {}", e));
                    }
                });
            }
            Err(err) => logger::error(err),
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
fn write_file(path: &str) -> TaurusResult<GeminiResponse> {
    let extension = path::Path::new(path)
        .extension()
        .unwrap_or_else(|| std::ffi::OsStr::new(""));

    let mime_type = match &*extension.to_string_lossy() {
        "gmi" => "text/gemini; charset=utf-8",
        ext => mime_guess::from_ext(ext)
            .first_raw()
            .unwrap_or("text/plain"),
    };

    match read_file(path) {
        Ok(buf) => Ok(GeminiResponse::success(buf, mime_type)),
        Err(err) => {
            // Cannot read file or it doesn't exist
            logger::error(format!("{}: {}", path, err));

            Ok(GeminiResponse::not_found())
        }
    }
}

fn handle_client(mut stream: TlsStream<TcpStream>, static_root: &str) -> TaurusResult<usize> {
    let mut buffer = [0; 1024];

    stream
        .read(&mut buffer)
        .map_err(TaurusError::StreamReadFailed)?;

    let raw_request = String::from_utf8_lossy(&buffer[..]).into_owned();

    let request = GeminiRequest::parse(&raw_request)?;
    let url_path = request.file_path();
    let file_path = path::Path::new(url_path);

    if file_path.has_root() {
        // File starts with `/` (*nix) or `\\` (Windows), decline it
        GeminiResponse::not_found().send(stream)
    } else {
        let path = path::Path::new(&static_root)
            .join(&file_path)
            .as_path()
            .to_owned();

        // Check if file/dir exists
        if path.exists() {
            // If it's a directory, try to find index.gmi
            if path.is_dir() {
                write_file(&path.join("index.gmi").to_string_lossy())?.send(stream)
            } else {
                write_file(&path.to_string_lossy())?.send(stream)
            }
        } else {
            GeminiResponse::not_found().send(stream)
        }
    }
}
