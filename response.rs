use std::fmt;

pub struct Response {
	body: ~str
}

impl fmt::Show for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf, "Response:({})", self.body)
    }
}
