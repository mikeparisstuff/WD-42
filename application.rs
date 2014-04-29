#![crate_id = "application"]
#![crate_type="lib"]

//! The main WD-42 application.
//!
//! The majority of the interface that you as a user will see is exposed
//! via this Application module and includes activities such as
//! registering request handlers, applying middleware functions, and setting
//! configuration variables.

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

/// The Application struct. The meat of the WD-42 Framework
#[deriving(Clone)]
pub struct App {
	/// The root directory for your static files including html, css, and js files.
    viewDirectory: Path,
    /// The data structure to hold your get routes and get request handlers.
    /// Users should not interact with this directly and should use App::get() instead.
	getRoutes: ~HashMap<~str, fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)>,
    /// The data structure to hold your put routes and put request handlers.
    /// Users should not interact with this directly and should use App::put() instead.
    putRoutes: ~HashMap<~str, fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)>,
    /// The data structure to hold your post routes and post request handlers.
    /// Users should not interact with this directly and should use App::post() instead.
    postRoutes: ~HashMap<~str, fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)>,
    /// The data structure to hold your del routes and del request handlers.
    /// Users should not interact with this directly and should use App::del() instead.
    delRoutes: ~HashMap<~str, fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)>,
    /// The data structure to hold the applied middleware functions.  Add a function to this list using App::apply()
    middlewareStack: ~Vec<fn(&mut http::server::request::Request)>,
    /// The port that your server will be listening on
    port: u16
}

impl App {

    /// Create a new Application.
    pub fn new() -> App {
        App {
            viewDirectory: os::getcwd(),
            getRoutes: ~HashMap::new(),
            postRoutes: ~HashMap::new(),
            delRoutes: ~HashMap::new(),
            putRoutes: ~HashMap::new(),
            middlewareStack: ~Vec::new(),
            port: 8000
        }
    }

    /// Set the port to be listened on
    pub fn setPort(&mut self, new_port: u16) {
        self.port = new_port
    }

    /// Supply a port to listen on and actually start the WD-42 server.
    pub fn listen(&mut self, port : u16) {
        self.setPort(port);
        let s = self.clone();
        s.serve_forever();
    }

    /// Add a middleware function to the middleware stack to be run on the
    /// Request objects before reaching your request handlers.
    ///
    /// This is often an ideal place to put authentication logic as well as
    /// light analytics tracking.
    pub fn apply(&mut self, f : fn(&mut http::server::request::Request)) {
        self.middlewareStack.push(f);
    }


    /// Set the root public directory where you will place your html, css, and js files
    pub fn set_public_dir(&mut self, path_to_dir: &str) {
        self.viewDirectory.push(Path::new(path_to_dir));
        println!("Setting public directory to: {}", self.viewDirectory.display());
    }

    /// Add a GET route and GET request handler for that route to the application.
    /// app.get("/", indexGet) will register the indexGet function to be called whenever
    /// the server sees a get request to the route "/"
    pub fn get(&mut self, route : ~str, f : fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)) {
        self.getRoutes.insert(route, f);
    }

    /// The same as app.get() except for post requests
    pub fn post(&mut self, route : ~str, f : fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)) {
        self.postRoutes.insert(route, f);
    }

    /// The same as app.get() except for put requests
    pub fn put(&mut self, route : ~str, f : fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)) {
        self.putRoutes.insert(route, f);
    }

    /// The same as app.get() except for del requests
    pub fn del(&mut self, route : ~str, f : fn(&http::server::request::Request, &mut http::server::response::ResponseWriter<>)) {
        self.delRoutes.insert(route, f);
    }
}

impl Server for App {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: self.port }, viewDirectory: self.viewDirectory.clone()}
    }

    fn handle_request(&self, r: &mut Request, w: &mut ResponseWriter) {
        w.headers.date = Some(time::now_utc());
        w.headers.server = Some(~"WD-42 Server");

        for mw in self.middlewareStack.iter() {
            (*mw)(r);
        }

        let uri = r.request_uri.clone();
        match (&r.method, uri) {
            (&Get, AbsolutePath(p)) => {
                println!("\nGET request to path: {}", p);
                // let s = self.clone();
                if  self.getRoutes.contains_key(&p) {
                    let v = self.getRoutes.get(&p);
                    // let f = v.get;
                    (*v)(r, w);
                } else {
                    let path : Path = self.viewDirectory.clone().join(Path::new(p.slice_from(1)));
                    if path.exists() {
                        let path = p.clone();
                        w.sendFile(path.slice_from(1).to_owned());
                    } else {
                        w.status = MethodNotAllowed;
                        w.write(bytes!("Page not found"));
                    }
                }
            },
            (&Post, AbsolutePath(p)) => {
                if self.postRoutes.contains_key(&p) {
                    println!("\nPOST request to path");
                    let v = self.postRoutes.get(&p);
                    // let f = v.get;
                    (*v)(r, w);
                } else {
                    w.status = MethodNotAllowed;
                    w.write(bytes!("Page not found"));
                }
            },
            (&Put, AbsolutePath(p)) => {
                if self.putRoutes.contains_key(&p) {
                    println!("\nPUT request to path");
                    let v = self.putRoutes.get(&p);
                    // let f = v.get;
                    (*v)(r, w);
                } else {
                    w.status = MethodNotAllowed;
                    w.write(bytes!("Page not found"));
                }
            },
            (&Delete, AbsolutePath(p)) => {
                if self.delRoutes.contains_key(&p) {
                    println!("\nDELETE request to path");
                    let v = self.delRoutes.get(&p);
                    // let f = v.get;
                    (*v)(r, w);
                } else {
                    w.status = MethodNotAllowed;
                    w.write(bytes!("Page not found"));
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
        printstr = printstr + "\n=================================\n\tRoutes defined for:\n=================================\nGET:";
        for (route, func) in self.getRoutes.iter() {
            printstr = printstr + format!("\n    |  {}", route);
        }
        printstr = printstr + "\n\nPOST:";
        for (route, func) in self.postRoutes.iter() {
            printstr = printstr + format!("\n    |  {}", route);
        }
        printstr = printstr + "\n\nPUT:";
        for (route, func) in self.putRoutes.iter() {
            printstr = printstr + format!("\n    |  {}", route);
        }
        printstr = printstr + "\n\nDELETE:";
        for (route, func) in self.delRoutes.iter() {
            printstr = printstr + format!("\n    |  {}", route);
        }
        write!(f.buf, "{}", printstr)
    }
}

fn main() {
	println!("Yay compilation");

}
