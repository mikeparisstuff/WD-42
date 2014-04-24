use collections::HashMap;
use std::fmt;

pub struct Request {
	header: HashMap<~str, ~str>,
	body: ~str
}

impl fmt::Show for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf, "Request:({}, {})", self.header, self.body)
    }
}
