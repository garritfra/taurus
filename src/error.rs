pub struct SimpleError(String);

impl std::convert::From<String> for SimpleError {
    fn from(string: String) -> Self {
        Self(string)
    }
}

impl std::fmt::Debug for SimpleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        formatter.write_str(&self.0)
    }
}
