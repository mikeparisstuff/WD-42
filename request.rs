use collections::HashMap;

pub struct Request {
	header: HashMap<~str, ~str>,
	body: ~str
}
