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

    pub fn file_path(&self) -> Option<&str> {
        self.path
            .path()
            .chars()
            .next()
            .map(|c| &self.path.path()[c.len_utf8()..])
    }
}

fn parse_path(req: &str) -> Option<&str> {
    req.split("\r\n").next()
}
