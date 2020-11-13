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
        let mut s = s.to_string();

        // Add gemini: scheme if not explicitly set
        if s.starts_with("//") {
            s = format!("gemini:{}", s);
        }

        // Check protocol
        if let Some(proto_end) = s.find("://") {
            // If set, check if it's allowed
            let protocol = &s[..proto_end];

            if protocol != "gemini" {
                // TODO: return 53 error instead of dropping
                return Err(TaurusError::InvalidRequest("invalid protocol".into()));
            }
        } else {
            // If no protocol is found, gemini: is implied
            s = format!("gemini://{}", s);
        }

        // Extract and parse the url from the request.
        let raw = s
            .trim_end_matches(0x0 as char)
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

    fn check_request(raw: &str, expected_url: &str) {
        let req = GeminiRequest::parse(raw).unwrap();

        assert_eq!(
            req,
            GeminiRequest {
                url: Url::parse(expected_url).unwrap()
            }
        );
    }

    #[test]
    fn parse_request() {
        check_request("gemini://example.space\r\n", "gemini://example.space");
    }

    #[test]
    fn parse_without_scheme() {
        check_request("example.space\r\n", "gemini://example.space");
    }

    #[test]
    fn parse_without_scheme_double_slash() {
        check_request("//example.space\r\n", "gemini://example.space");
    }

    #[test]
    fn parse_malformed_request() {
        let raw = "gemini://example.space";

        match GeminiRequest::parse(raw) {
            Err(TaurusError::InvalidRequest(_)) => {}
            x => panic!("expected TaurusError::InvalidRequest, got: {:?}", x),
        }
    }
}
