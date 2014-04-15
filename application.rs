#![feature(globs)]

extern crate collections;

use std::io::*;
use collections::HashMap;

mod request;
mod response;

// The basic Rust App to be exposed
struct App {
	getRoutes: ~HashMap<~str, int>,
	postRoutes: ~HashMap<~str, int>,
	putRoutes: ~HashMap<~str, int>,
	delRoutes: ~HashMap<~str, int>
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
	fn get(&mut self, route : &str, function: | req: &request::Request, res: &response::Response| -> () ) -> () {
		// Add this route -> function pair in our routing datastructure

	}

	fn post(&mut self, route: &str, function: | req: &request::Request, res: &response::Response| -> () ) -> () {
		// Add this route -> function pair to our routing datastructure

	}

	fn put(&mut self, route: &str, function: | req: &request::Request, res: &response::Response| -> () ) -> () {
		// Add this route -> function pair to our routing datastructure
	}

	fn del(&mut self, route: &str, function: | req: &request::Request, res: &response::Response| -> () ) -> () {
		// Add this route -> function pair to our routing datastructure
	}
}

fn main() {
	println!("Yay compilation");
}