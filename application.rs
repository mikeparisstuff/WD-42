#![feature(globs)]

extern crate collections;
extern crate http;
extern crate time;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::Writer;

use http::server::{Config, Server, Request, ResponseWriter};
use http::server::request::{Star, AbsoluteUri, AbsolutePath, Authority};
use http::status::{BadRequest, MethodNotAllowed};
use http::method::{Get, Head, Post, Put, Delete, Trace, Options, Connect, Patch};
use http::headers::content_type::MediaType;
use std::fmt;

use std::io::*;
use collections::HashMap;

mod request;
mod response;
pub mod view;

// The basic Rust App to be exposed

#[deriving(Clone)]
pub struct App {
	// TODO: Change Request/Response objects to work with rust-http
	routes: ~HashMap<~str, view::View>,
    port: u16
}

impl App {
    pub fn new() -> App {
        App {
            routes: ~HashMap::new(),
            port: 8000
        }
    }

	/*
	*	Setup routing functions
	*/
	// map a route string to a function to handle that route
    pub fn setRoutes(&mut self, new_routes: ~HashMap<~str, view::View>) -> () {
        self.routes = new_routes
    }

    pub fn setPort(&mut self, new_port: u16) {
        self.port = new_port
    }

    pub fn listen(&mut self, port : u16) {
        self.setPort(port);
        let s = self.clone();
        s.serve_forever();
    }
}

impl Server for App {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: self.port }}
    }

    fn handle_request(&self, r: &Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());
        w.headers.server = Some(~"Rustic Server Hold Mah Dick");

        match (&r.method, &r.request_uri) {
            (&Get, &AbsolutePath(_)) => {
                println!("GET request to path");
                let v : &view::View = self.routes.get(&~"/");
                let f = v.get;
                f(r, w);

            },
            // (&Post, &AbsolutePath(path)) => {

            // },
            // (&Put, &AbsolutePath(path)) => {

            // },
            // (&Delete, &AbsolutePath(path)) => {

            // }
            (_, _) => {
                println!("Could not match with a predefined request handler");
            }
        }
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
