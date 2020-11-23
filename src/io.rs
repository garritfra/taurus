use crate::error::{TaurusError, TaurusResult};
use native_tls::Identity;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Read a file into Vec
pub fn read_file(file_path: &str) -> Result<Vec<u8>, io::Error> {
    let mut file = File::open(file_path)?;
    let mut buf = Vec::new();

    file.read_to_end(&mut buf)?;

    Ok(buf)
}

/// Read certificate file
pub fn load_cert(cert_file: &str, password: &str) -> TaurusResult<Identity> {
    let identity = read_file(&cert_file).map_err(TaurusError::NoIdentity)?;
    let identity = Identity::from_pkcs12(&identity, &password)?;

    Ok(identity)
}

/// Resolve path to a file, returning index.gmi if a subdirectory is encountered
///
/// If path points to a file, it is returned.
/// If path points to a directory, `./index.gmi` is returned
pub fn resolve_path(path: &Path) -> String {
    if path.is_dir() {
        path.join("index.gmi").to_string_lossy().into_owned()
    } else {
        path.to_string_lossy().into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_path_file() {
        let path = Path::new("./file.gmi");

        assert_eq!(resolve_path(&path), String::from("./file.gmi"));
    }

    #[test]
    fn resolve_path_dir() {
        let path = Path::new("./");

        assert_eq!(resolve_path(&path), String::from("./index.gmi"));
    }
}
