#![feature(globs)]

extern crate collections;
extern crate http;

use std::io::*;
use collections::HashMap;

mod request;
mod response;

// The basic Rust App to be exposed
struct App {
	getRoutes: ~HashMap<~str, fn(req: &request::Request, res: &response::Response)>,
	postRoutes: ~HashMap<~str, fn(req: &request::Request, res: &response::Response)>,
	putRoutes: ~HashMap<~str, fn(req: &request::Request, res: &response::Response)>,
	delRoutes: ~HashMap<~str, fn(req: &request::Request, res: &response::Response)>
}

impl App {
	fn new() -> App{

		App {
			getRoutes: ~HashMap::new(),
			postRoutes: ~HashMap::new(),
			putRoutes: ~HashMap::new(),
			delRoutes: ~HashMap::new()
		}
	}


	/*
	*	Setup routing functions
	*/
	// map a route string to a function to handle that route
	fn get(&mut self, route : &str, function: fn(req: &request::Request, res: &response::Response) -> () ) -> () {
		// Add this route -> function pair in our routing datastructure
		self.getRoutes.find_or_insert(route.to_owned(), function);

	}

	fn post(&mut self, route: &str, function: fn(req: &request::Request, res: &response::Response) -> () ) -> () {
		// Add this route -> function pair to our routing datastructure
		self.postRoutes.find_or_insert(route.to_owned(), function);
	}

	fn put(&mut self, route: &str, function: fn(req: &request::Request, res: &response::Response) -> () ) -> () {
		// Add this route -> function pair to our routing datastructure
		self.putRoutes.find_or_insert(route.to_owned(), function);
	}

	fn del(&mut self, route: &str, function: fn(req: &request::Request, res: &response::Response) -> () ) -> () {
		// Add this route -> function pair to our routing datastructure
		self.delRoutes.find_or_insert(route.to_owned(), function);
	}
}

fn main() {
	println!("Yay compilation");
}
