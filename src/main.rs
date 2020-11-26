extern crate native_tls;
extern crate url;

mod config;
mod error;
mod gemini;
mod io;
mod logger;

use error::{TaurusError, TaurusResult};
use gemini::{GeminiRequest, GeminiResponse};
use native_tls::{TlsAcceptor, TlsStream};
use std::{
    io::Read,
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
    let identity = crate::io::load_cert(&cert_file, &config.certificate_password)?;

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

/// Send file as a response
fn handle_client(mut stream: TlsStream<TcpStream>, static_root: &str) -> TaurusResult<usize> {
    let mut buffer = [0; 1024];

    stream
        .read(&mut buffer)
        .map_err(TaurusError::StreamReadFailed)?;

    let raw_request = String::from_utf8(buffer.to_vec())?;

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
            GeminiResponse::from_file(&crate::io::resolve_path(&path))?.send(stream)
        } else {
            GeminiResponse::not_found().send(stream)
        }
    }
}
