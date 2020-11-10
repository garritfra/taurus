use native_tls::TlsStream;
use std::io::Write;
use std::net::TcpStream;
use url::Url;

pub struct GeminiRequest {
    path: Url,
}

impl GeminiRequest {
    pub fn from_string(request: &str) -> Result<Self, String> {
        let gemini_request = GeminiRequest {
            path: Url::parse(&parse_path(request).ok_or("Invalid path")?.to_string())
                .map_err(|e| e.to_string())?,
        };

        Ok(gemini_request)
    }

    /// Get file path
    pub fn file_path(&self) -> &str {
        self.path
            .path()
            .chars()
            .next()
            .map_or("", |c| &self.path.path()[c.len_utf8()..])
    }
}

fn parse_path(req: &str) -> Option<&str> {
    req.split("\r\n").next()
}

pub struct GeminiResponse {
    pub status: [u8; 2],
    pub meta: Vec<u8>,
    pub body: Option<Vec<u8>>,
}

impl GeminiResponse {
    pub fn success(body: Vec<u8>) -> Self {
        GeminiResponse {
            status: [b'2', b'0'],
            meta: b"text/gemini; charset=utf-8".to_vec(),
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

    pub fn send(&self, mut stream: TlsStream<TcpStream>) -> Result<usize, String> {
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

        stream
            .write(&buf)
            .map_err(|e| format!("could not write to stream: {}", e))
    }
}
