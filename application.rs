#![feature(globs)]

extern crate collections;
extern crate http;
use http::server::{Request, ResponseWriter};
use http::headers;
use std::fmt;

use std::io::*;
use collections::HashMap;

mod request;
mod response;
pub mod view;

// The basic Rust App to be exposed
pub struct App {
	// TODO: Change Request/Response objects to work with rust-http
	routes: ~HashMap<~str, view::View>,
    port: ~u16
}

impl App {
    pub fn new() -> App {
        App {
            routes: ~HashMap::new(),
            port: ~8080
        }
    }

	/*
	*	Setup routing functions
	*/
	// map a route string to a function to handle that route
    pub fn setRoutes(&mut self, new_routes: ~HashMap<~str, view::View>) -> () {
        self.routes = new_routes
    }

    pub fn setPort(&mut self, new_port: ~u16) {
        self.port = new_port
    }
}

// So we can println!("{}", myApp)
impl fmt::Show for App {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut printstr: ~str = format!("Rustic app running on port: {}", self.port);
        printstr = printstr + "\n\tRoutes defined for: ";
        for (route, func) in self.routes.iter() {
            printstr = printstr + format!("\n\t{}", route);
        }
        write!(f.buf, "{}", printstr)
    }
}

fn main() {
	println!("Yay compilation");
}
