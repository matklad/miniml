
#[derive(Debug)]
pub struct ParseError {
    location: usize,
    message: String,
}

impl ParseError {
    pub fn new(location: usize, message: String) -> ParseError {
        ParseError {
            location: location,
            message: message,
        }
    }
}