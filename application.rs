#![feature(globs)]

extern crate collections;
extern crate http;
use http::server::{Request, ResponseWriter};
use http::headers;

use std::io::*;
use collections::HashMap;

mod request;
mod response;

// The basic Rust App to be exposed
struct App {
	// TODO: Change Request/Response objects to work with rust-http
	routes: ~HashMap<~str, fn(req: &request::Request, res: &response::Response)>,
    port: ~u16
}

impl App {
    fn new() -> App {
        App {
            routes: ~HashMap::new(),
            port: ~8080
        }
    }

	/*
	*	Setup routing functions
	*/
	// map a route string to a function to handle that route
    fn setRoutes(&mut self, new_routes: ~HashMap<~str, fn(req: &request::Request, res: &response::Response)>) -> () {
        self.routes = new_routes
    }

    fn setPort(&mut self, new_port: ~u16) {
        self.port = new_port
    }
}

fn main() {
	println!("Yay compilation");
}
