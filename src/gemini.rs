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

    fn unsafe_file_path(&self) -> Option<&str> {
        self.path
            .path()
            .chars()
            .next()
            .map(|c| &self.path.path()[c.len_utf8()..])
    }

    pub fn file_path(&self) -> Option<&str> {
        match self.unsafe_file_path() {
            Some(path) if path.contains("..") || path.starts_with("/") => None,
            Some(path) => Some(path),
            None => None,
        }
    }
}

fn parse_path(req: &str) -> Option<&str> {
    req.split("\r\n").next()
}

pub struct GeminiResonse {
    pub status: [u8; 2],
    pub meta: Vec<u8>,
    pub body: Option<Vec<u8>>,
}

impl GeminiResonse {
    pub fn new() -> Self {
        GeminiResonse {
            status: [b'2', b'0'],
            meta: "text/gemini; charset=utf-8".as_bytes().to_vec(),
            body: None,
        }
    }

    pub fn build(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

        // 20 SUCESS status
        buf.extend(&self.status);

        // <Space>
        buf.push(0x20);

        // <Meta>
        buf.extend(&self.meta);

        buf.extend("\r\n".as_bytes());

        if let Some(body) = &self.body {
            buf.extend(body);
        }

        buf
    }
}
