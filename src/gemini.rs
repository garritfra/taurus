use crate::error::{TaurusError, TaurusResult};
use native_tls::TlsStream;
use std::{io::Write, net::TcpStream, str::FromStr};
use url::Url;

#[derive(Debug, PartialEq, Eq)]
pub struct GeminiRequest {
    url: Url,
}

impl GeminiRequest {
    pub fn parse(request: &str) -> TaurusResult<Self> {
        Self::from_str(request)
    }

    /// Get file path
    pub fn file_path(&self) -> &str {
        self.url
            .path()
            .chars()
            .next()
            .map_or("", |c| &self.url.path()[c.len_utf8()..])
    }
}

impl FromStr for GeminiRequest {
    type Err = TaurusError;

    fn from_str(s: &str) -> TaurusResult<Self> {
        // Extract and parse the url from the request.
        let raw = s
            .strip_suffix("\r\n")
            .ok_or_else(|| TaurusError::InvalidRequest("malformed request".into()))?;
        let url = Url::parse(&raw)
            .map_err(|e| TaurusError::InvalidRequest(format!("invalid url: {}", e)))?;

        Ok(Self { url })
    }
}

pub struct GeminiResponse {
    pub status: [u8; 2],
    pub meta: Vec<u8>,
    pub body: Option<Vec<u8>>,
}

impl GeminiResponse {
    pub fn success(body: Vec<u8>, mime_type: &str) -> Self {
        GeminiResponse {
            status: [b'2', b'0'],
            meta: mime_type.as_bytes().to_vec(),
            body: Some(body),
        }
    }

    pub fn not_found() -> Self {
        GeminiResponse {
            status: [b'5', b'1'],
            meta: "Resource not found".into(),
            body: None,
        }
    }

    pub fn send(&self, mut stream: TlsStream<TcpStream>) -> TaurusResult<usize> {
        let mut buf: Vec<u8> = Vec::new();

        // <Status>
        buf.extend(&self.status);

        // <Space>
        buf.push(0x20);

        // <Meta>
        buf.extend(&self.meta);

        buf.extend(b"\r\n");

        if let Some(body) = &self.body {
            buf.extend(body);
        }

        stream.write(&buf).map_err(TaurusError::StreamWriteFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_request() {
        let raw = "gemini://example.space\r\n";

        let req = GeminiRequest::parse(raw).unwrap();
        assert_eq!(
            req,
            GeminiRequest {
                url: Url::parse("gemini://example.space").unwrap()
            }
        );
    }

    #[test]
    fn parse_malformed_request() {
        let raw = "gemini://example.space";

        match GeminiRequest::parse(raw) {
            Err(TaurusError::InvalidRequest(_)) => {}
            x => panic!("expected TaurusError::InvalidRequest, got: {:?}", x),
        }
    }

    #[test]
    fn parse_invalid_request_url() {
        let raw = "foobar@example.com\r\n";

        match GeminiRequest::parse(raw) {
            Err(TaurusError::InvalidRequest(_)) => {}
            x => panic!("expected TaurusError::InvalidRequest, got: {:?}", x),
        }
    }
}
