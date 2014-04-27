#![feature(globs)]

extern crate collections;
extern crate http;
extern crate time;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::{Writer, File};
use std::path::Path;
use std::{fmt, Vec, os};

use http::server::{Config, Server, Request, ResponseWriter};
use http::server::request::{Star, AbsoluteUri, AbsolutePath, Authority};
use http::status::{BadRequest, MethodNotAllowed, Ok, InternalServerError};
use http::method::{Get, Head, Post, Put, Delete, Trace, Options, Connect, Patch};
use http::headers::content_type::MediaType;

use collections::HashMap;

// The basic Rust App to be exposed

#[deriving(Clone)]
pub struct App {
	// TODO: Change Request/Response objects to work with rust-http
    viewDirectory: ~str,
	getRoutes: ~HashMap<~str, fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)>,
    putRoutes: ~HashMap<~str, fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)>,
    postRoutes: ~HashMap<~str, fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)>,
    delRoutes: ~HashMap<~str, fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)>,
    middlewareStack: ~Vec<fn(&http::server::request::Request)>,
    port: u16
}

impl App {
    pub fn new() -> App {
        App {
            viewDirectory: os::getcwd().as_str().unwrap().clone().to_owned(),
            getRoutes: ~HashMap::new(),
            postRoutes: ~HashMap::new(),
            delRoutes: ~HashMap::new(),
            putRoutes: ~HashMap::new(),
            middlewareStack: ~Vec::new(),
            port: 8000
        }
    }

	/*
	*	Setup routing functions
	*/
	// map a route string to a function to handle that route
    // pub fn setRoutes(&mut self, new_routes: ~HashMap<~str, fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)>) -> () {
    //     self.routes = new_routes
    // }

    pub fn setPort(&mut self, new_port: u16) {
        self.port = new_port
    }

    pub fn listen(&mut self, port : u16) {
        self.setPort(port);
        let s = self.clone();
        s.serve_forever();
    }

    pub fn apply(&mut self, f : fn(&http::server::request::Request)) {
        println!("Adding function to middleware");
        self.middlewareStack.push(f);
    }
    /*
    *   Interface for user to send html and other files
    */
    pub fn set_view_dir(&mut self, path_to_dir: &str) {
        println!("CWD: {}", os::getcwd().display());
        // println!("Rel Path: {}", Path::new(path_to_dir).display());
        let mut viewDir = Path::new(self.viewDirectory.clone());
        viewDir.push(Path::new(path_to_dir));
        self.viewDirectory = viewDir.as_str().unwrap().clone().to_owned();
        // println!("Total path: {}", self.viewDirectory.display());
    }


    /*
    *   Interface for user to connect functions to handle different HTTP methods at different endpoints
    */
    pub fn get(&mut self, route : ~str, f : fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)) {
        self.getRoutes.insert(route, f);
    }

    pub fn post(&mut self, route : ~str, f : fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)) {
        self.postRoutes.insert(route, f);
    }

    pub fn put(&mut self, route : ~str, f : fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)) {
        self.putRoutes.insert(route, f);
    }

    pub fn del(&mut self, route : ~str, f : fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)) {
        self.delRoutes.insert(route, f);
    }
}

impl Server for App {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: self.port }, viewDirectory: self.viewDirectory.clone()}
    }

    fn handle_request(&self, r: &Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());
        w.headers.server = Some(~"Rustic Server Hold Mah Dick");

        for mw in self.middlewareStack.iter() {
            (*mw)(r);
        }

        let uri = r.request_uri.clone();
        match (&r.method, uri) {
            (&Get, AbsolutePath(p)) => {
                println!("GET request to path: {}", p);
                if  self.getRoutes.contains_key(&p) {
                    let v = self.getRoutes.get(&p);
                    // let f = v.get;
                    (*v)(r, w);
                } else {
                    w.status = MethodNotAllowed;

                }
            },
            (&Post, AbsolutePath(p)) => {
                if self.postRoutes.contains_key(&p) {
                    println!("POST request to path");
                    let v = self.postRoutes.get(&p);
                    // let f = v.get;
                    (*v)(r, w);
                } else {
                    w.status = MethodNotAllowed;
                }
            },
            (&Put, AbsolutePath(p)) => {
                if self.putRoutes.contains_key(&p) {
                    println!("PUT request to path");
                    let v = self.putRoutes.get(&p);
                    // let f = v.get;
                    (*v)(r, w);
                } else {
                    w.status = MethodNotAllowed;
                }
            },
            (&Delete, AbsolutePath(p)) => {
                if self.delRoutes.contains_key(&p) {
                    println!("DELETE request to path");
                    let v = self.delRoutes.get(&p);
                    // let f = v.get;
                    (*v)(r, w);
                } else {
                    w.status = MethodNotAllowed;
                }
            },
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
        for (route, func) in self.getRoutes.iter() {
            printstr = printstr + format!("\n\t{}", route);
        }
        write!(f.buf, "{}", printstr)
    }
}

fn main() {
	println!("Yay compilation");

}
